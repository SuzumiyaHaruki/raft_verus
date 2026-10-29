use std::convert::TryFrom;
use super::msg::*;
use crate::kvraft::client::ClerkCore;
use std::{collections::HashMap, net::SocketAddr};

pub struct Clerk {
    core: std::sync::Arc<ClerkCore<Op, Option<Config>>>,
    transport:madsim::runtime::NodeHandle,
}

impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Clerk {
        let id=crate::kvraft::client::CLIENT_ID.fetch_update(std::sync::atomic::Ordering::SeqCst,std::sync::atomic::Ordering::SeqCst,|v|v.checked_add(1)).expect("client IDs exhausted");
        let number=u32::try_from(id).expect("client addresses exhausted");
        let transport=madsim::runtime::Handle::current().create_node().ip(std::net::Ipv4Addr::from(0xc6120000u32.checked_add(number).expect("client addresses exhausted")).into()).build();
        Clerk {
            core: std::sync::Arc::new(ClerkCore::new(servers)),transport,
        }
    }

    async fn call(&self,args:Op)->Option<Config> {let core=self.core.clone();self.transport.spawn(async move {core.call(args).await}).await.expect("controller client transport")}

    pub async fn query(&self) -> Config {
        self.call(Op::Query { num: u64::MAX }).await.unwrap()
    }

    pub async fn query_at(&self, num: u64) -> Config {
        self.call(Op::Query { num }).await.unwrap()
    }

    pub async fn join(&self, groups: HashMap<Gid, Vec<SocketAddr>>) {
        self.call(Op::Join { groups }).await;
    }

    pub async fn leave(&self, gids: &[u64]) {
        self.call(Op::Leave { gids: gids.into() }).await;
    }

    pub async fn move_(&self, shard: usize, gid: u64) {
        self.call(Op::Move { shard, gid }).await;
    }
}
