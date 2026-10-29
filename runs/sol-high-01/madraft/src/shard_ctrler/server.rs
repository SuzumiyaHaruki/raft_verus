use super::{msg::*, N_SHARDS};
use crate::kvraft::server::{Server, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

pub type ShardCtrler = Server<ShardInfo>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardInfo { configs: Vec<Config> }
impl Default for ShardInfo { fn default() -> Self { Self { configs: vec![Config::default()] } } }

fn rebalance(config: &mut Config) {
    if config.groups.is_empty() { config.shards.fill(0); return; }
    let mut loads: BTreeMap<u64, Vec<usize>> = config.groups.keys().map(|g| (*g, vec![])).collect();
    let mut free = vec![];
    for (i, gid) in config.shards.iter().enumerate() {
        if let Some(shards) = loads.get_mut(gid) { shards.push(i); } else { free.push(i); }
    }
    // Give the remainder slots to groups that already own the most shards.
    // This retains as many assignments as possible, with stable ID tie-breaking.
    let mut order: Vec<_> = loads.keys().copied().collect();
    order.sort_by_key(|gid| (std::cmp::Reverse(loads[gid].len()), *gid));
    let mut target = BTreeMap::new();
    for (rank, gid) in order.iter().enumerate() {
        target.insert(*gid, N_SHARDS / order.len() + usize::from(rank < N_SHARDS % order.len()));
    }
    for (gid, shards) in loads.iter_mut() {
        while shards.len() > target[gid] { free.push(shards.pop().unwrap()); }
    }
    free.sort_unstable();
    let mut free = free.into_iter();
    for (gid, shards) in loads.iter_mut() {
        while shards.len() < target[gid] { shards.push(free.next().unwrap()); }
        for &shard in shards.iter() { config.shards[shard] = *gid; }
    }
}
impl State for ShardInfo {
    type Command = Op;
    type Output = Option<Config>;
    fn apply(&mut self, cmd: Op) -> Option<Config> {
        if let Op::Query { num } = cmd {
            return Some(self.configs.get(usize::try_from(num).unwrap_or(usize::MAX))
                .unwrap_or_else(|| self.configs.last().unwrap()).clone());
        }
        let mut config = self.configs.last().unwrap().clone();
        let next = match config.num.checked_add(1) { Some(n) => n, None => return None };
        match cmd {
            Op::Join { groups } => { for (gid, addrs) in groups { if gid != 0 { config.groups.insert(gid, addrs); } } rebalance(&mut config); }
            Op::Leave { gids } => { for gid in gids { config.groups.remove(&gid); } rebalance(&mut config); }
            Op::Move { shard, gid } => { if shard >= N_SHARDS || !config.groups.contains_key(&gid) { return None; } config.shards[shard] = gid; }
            Op::Query { .. } => unreachable!(),
        }
        config.num = next;
        self.configs.push(config);
        None
    }
}
