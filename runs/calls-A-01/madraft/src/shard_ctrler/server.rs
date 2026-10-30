use super::msg::*;
use crate::kvraft::server::{Server, State};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub type ShardCtrler = Server<ShardInfo>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardInfo { configs: Vec<Config> }
impl Default for ShardInfo {
    fn default() -> Self { Self { configs: vec![Config::default()] } }
}
impl ShardInfo {
    fn rebalance(config: &mut Config) {
        let mut desired = BTreeMap::new();
        let mut gids: Vec<_> = config.groups.keys().copied().collect();
        gids.sort_unstable();
        if gids.is_empty() { config.shards.fill(0); return; }
        for (i, gid) in gids.iter().enumerate() {
            desired.insert(*gid, super::N_SHARDS / gids.len() + usize::from(i < super::N_SHARDS % gids.len()));
        }
        let mut counts = BTreeMap::new();
        for gid in config.shards { *counts.entry(gid).or_insert(0usize) += 1; }
        let mut free = vec![];
        for (index, gid) in config.shards.iter().enumerate() {
            let count = counts.get_mut(gid).unwrap();
            if *count > desired.get(gid).copied().unwrap_or(0) { *count -= 1; free.push(index); }
        }
        for (gid, target) in desired {
            let count = counts.entry(gid).or_insert(0);
            while *count < target { config.shards[free.pop().unwrap()] = gid; *count += 1; }
        }
    }
}
impl State for ShardInfo {
    type Command = Op;
    type Output = Option<Config>;
    fn apply(&mut self, cmd: Op) -> Self::Output {
        let mut config = self.configs.last().unwrap().clone();
        match cmd {
            Op::Query { num } => return Some(self.configs.get(num as usize).unwrap_or(&config).clone()),
            Op::Join { groups } => {
                for (gid, servers) in groups { if gid != 0 { config.groups.insert(gid, servers); } }
                Self::rebalance(&mut config);
            }
            Op::Leave { gids } => {
                for gid in gids { config.groups.remove(&gid); }
                Self::rebalance(&mut config);
            }
            Op::Move { shard, gid } => { if shard < super::N_SHARDS { config.shards[shard] = gid; } }
        }
        config.num = self.configs.len() as u64;
        self.configs.push(config);
        None
    }
}
