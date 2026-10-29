use super::{key2shard, msg::*};
use crate::{kvraft::client::Request, shard_ctrler::{client::Clerk as CtrlerClerk, msg::Config}};
use futures::lock::Mutex;
use madsim::{net, rand::{self, Rng}, time::*};
use std::net::SocketAddr;

pub struct Clerk { ctrl_ck: CtrlerClerk, client: u128, state: Mutex<(u64, Config)> }
impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        Self { ctrl_ck: CtrlerClerk::new(servers), client: rand::thread_rng().gen(), state: Mutex::new((0, Config::default())) }
    }
    async fn call(&self, operation: Op, key: &str) -> Reply {
        let mut state = self.state.lock().await;
        state.0 = state.0.checked_add(1).expect("client sequence exhausted");
        let request = Request { client: self.client, sequence: state.0, command: operation };
        let net = net::Endpoint::bind("0.0.0.0:0").await.expect("bind sharded client");
        let shard = key2shard(key);
        loop {
            let config = &state.1;
            let gid = config.shards[shard];
            if let Some(peers) = config.groups.get(&gid) {
                for &peer in peers {
                    match net.call_timeout(peer, request.clone(), Duration::from_millis(650)).await {
                        Ok(Ok(Reply::WrongGroup)) => break,
                        Ok(Ok(reply)) => return reply,
                        _ => {}
                    }
                }
            }
            sleep(Duration::from_millis(40)).await;
            state.1 = self.ctrl_ck.query().await;
        }
    }
    pub async fn get(&self, key: String) -> String {
        match self.call(Op::Get { key: key.clone() }, &key).await {
            Reply::Get { value } => value.unwrap_or_default(), _ => unreachable!("get reply"),
        }
    }
    pub async fn put(&self, key: String, value: String) { self.call(Op::Put { key: key.clone(), value }, &key).await; }
    pub async fn append(&self, key: String, value: String) { self.call(Op::Append { key: key.clone(), value }, &key).await; }
}
