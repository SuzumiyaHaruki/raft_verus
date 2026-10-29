use super::msg::*;
use crate::kvraft::server::{Server, State};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap,convert::TryFrom};
pub type ShardCtrler = Server<ShardInfo>;
#[derive(Debug,Default,Serialize,Deserialize)]
pub struct ShardInfo { configs:Vec<Config> }
impl ShardInfo {
    fn balance(config:&mut Config) {
        let mut groups:BTreeMap<u64,Vec<usize>>=config.groups.keys().map(|g|(*g,Vec::new())).collect();
        if groups.is_empty() {config.shards.fill(0);return;}
        let mut pool=Vec::new();
        for (shard,gid) in config.shards.iter().enumerate() {
            if let Some(shards)=groups.get_mut(gid) {shards.push(shard);} else {pool.push(shard);}
        }
        let count=groups.len();
        for (ordinal,(_,shards)) in groups.iter_mut().enumerate() {
            let target=super::N_SHARDS/count+usize::from(ordinal<super::N_SHARDS%count);
            while shards.len()>target {pool.push(shards.pop().unwrap());}
        }
        pool.sort_unstable();
        let mut next=pool.into_iter();
        for (ordinal,(gid,shards)) in groups.iter_mut().enumerate() {
            let target=super::N_SHARDS/count+usize::from(ordinal<super::N_SHARDS%count);
            while shards.len()<target {shards.push(next.next().unwrap());}
            for shard in shards {config.shards[*shard]=*gid;}
        }
    }
}
impl State for ShardInfo {
    type Command=Op;type Output=Option<Config>;
    fn apply(&mut self,cmd:Op)->Option<Config> {
        eprintln!("CTRL apply {:?}",cmd);
        if self.configs.is_empty() {self.configs.push(Config::default());}
        if let Op::Query{num}=cmd {
            return Some(self.configs.get(usize::try_from(num).unwrap_or(usize::MAX)).unwrap_or(self.configs.last().unwrap()).clone());
        }
        let mut config=self.configs.last().unwrap().clone();
        config.num=config.num.checked_add(1).expect("configuration exhausted");
        match cmd {
            Op::Join{groups}=>{for(gid,servers)in groups {if gid!=0 {config.groups.insert(gid,servers);}}Self::balance(&mut config);},
            Op::Leave{gids}=>{for gid in gids {config.groups.remove(&gid);}Self::balance(&mut config);},
            Op::Move{shard,gid}=>{if shard<super::N_SHARDS {config.shards[shard]=gid;}},
            Op::Query{..}=>unreachable!(),
        }
        self.configs.push(config);None
    }
}
