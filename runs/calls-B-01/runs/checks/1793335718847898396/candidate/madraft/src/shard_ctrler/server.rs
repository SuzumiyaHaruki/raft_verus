use super::{msg::*, N_SHARDS};
use crate::kvraft::server::{Server, State};
use serde::{Deserialize, Serialize};

pub type ShardCtrler = Server<ShardInfo>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardInfo { configs: Vec<Config> }
impl Default for ShardInfo { fn default() -> Self { Self { configs: vec![Config::default()] } } }

impl ShardInfo {
    fn rebalance(c: &mut Config) {
        let mut gids: Vec<_> = c.groups.keys().copied().collect();
        gids.sort_unstable();
        if gids.is_empty() { c.shards.fill(0); return; }
        let mut targets = std::collections::BTreeMap::new();
        // Give surplus shards to groups already holding more, minimizing moves.
        let mut by_count = gids.clone();
        by_count.sort_by_key(|gid| (std::cmp::Reverse(c.shards.iter().filter(|g| *g == gid).count()), *gid));
        for (i, gid) in by_count.into_iter().enumerate() {
            targets.insert(gid, N_SHARDS/gids.len() + usize::from(i < N_SHARDS%gids.len()));
        }
        let mut held = std::collections::BTreeMap::<u64, usize>::new();
        let mut free = Vec::new();
        for (shard, gid) in c.shards.iter().enumerate() {
            let n = held.entry(*gid).or_default();
            if targets.get(gid).map_or(false, |target| *n < *target) { *n += 1; }
            else { free.push(shard); }
        }
        let mut free = free.into_iter();
        for gid in gids {
            let have = held.get(&gid).copied().unwrap_or(0);
            for _ in have..targets[&gid] { c.shards[free.next().unwrap()] = gid; }
        }
    }
}
impl State for ShardInfo {
    type Command = Op;
    type Output = Option<Config>;
    fn apply(&mut self, cmd: Op) -> Self::Output {
        if let Op::Query { num } = cmd {
            return Some(self.configs.get(num as usize).unwrap_or_else(|| self.configs.last().unwrap()).clone());
        }
        let mut c = self.configs.last().unwrap().clone();
        c.num = c.num.checked_add(1).expect("configuration exhausted");
        match cmd {
            Op::Join { groups } => { for (gid, servers) in groups { if gid != 0 { c.groups.insert(gid, servers); } } Self::rebalance(&mut c); }
            Op::Leave { gids } => { for gid in gids { c.groups.remove(&gid); } Self::rebalance(&mut c); }
            Op::Move { shard, gid } => { if shard < N_SHARDS && c.groups.contains_key(&gid) { c.shards[shard] = gid; } }
            Op::Query { .. } => unreachable!(),
        }
        self.configs.push(c);
        None
    }
}
