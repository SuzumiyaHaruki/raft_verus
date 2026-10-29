use super::msg::*;
use crate::kvraft::client::ClerkCore;
use std::{collections::HashMap, net::SocketAddr};

pub struct Clerk {
    core: ClerkCore<Op, Option<Config>>,
}

impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Clerk {
        // The lab controller clerk may run directly on MadSim's root node.
        // Unlike deployed peers, that node is constructed without an IP.
        // This address is reserved for the lab client transport, not a member.
        if let Some(root)=madsim::runtime::Handle::current().get_node("madsim-main") {
            madsim::net::NetSim::current().set_ip(root.id(),"198.18.0.1".parse().unwrap());
        }
        Clerk {
            core: ClerkCore::new(servers),
        }
    }

    pub async fn query(&self) -> Config {
        self.core.call(Op::Query { num: u64::MAX }).await.unwrap()
    }

    pub async fn query_at(&self, num: u64) -> Config {
        self.core.call(Op::Query { num }).await.unwrap()
    }

    pub async fn join(&self, groups: HashMap<Gid, Vec<SocketAddr>>) {
        self.core.call(Op::Join { groups }).await;
    }

    pub async fn leave(&self, gids: &[u64]) {
        self.core.call(Op::Leave { gids: gids.into() }).await;
    }

    pub async fn move_(&self, shard: usize, gid: u64) {
        self.core.call(Op::Move { shard, gid }).await;
    }
}
