use super::msg::*;
use crate::{kvraft::client::ClerkCore, shard_ctrler::{client::Clerk as CtrlerClerk, msg::Config}};
use std::{net::SocketAddr, sync::{Arc, Mutex}, collections::HashMap};
use madsim::{rand::Rng, time::{sleep, Duration}};
pub struct Clerk {
    ctrl_ck: CtrlerClerk,
    config: Mutex<Config>,
    groups: Mutex<HashMap<u64, Arc<ClerkCore<Op, Reply>>>>,
    client: u64,
    sequence: futures::lock::Mutex<u64>,
}
impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        Self { ctrl_ck: CtrlerClerk::new(servers), config: Mutex::new(Config::default()), groups: Mutex::new(HashMap::new()),
            client: madsim::rand::thread_rng().gen(), sequence: futures::lock::Mutex::new(0) }
    }
    async fn call(&self, action: Action) -> Reply {
        let mut seq = self.sequence.lock().await;
        *seq = crate::kvraft::sequence::next_sequence(*seq).expect("client exhausted");
        let shard = super::key2shard(action.key());
        let op = Op::Request { client: self.client, sequence: *seq, action };
        loop {
            let core = {
                let config = self.config.lock().unwrap();
                let gid = config.shards[shard];
                config.groups.get(&gid).filter(|servers| !servers.is_empty()).map(|servers| {
                    self.groups.lock().unwrap().entry(gid).or_insert_with(|| Arc::new(ClerkCore::new(servers.clone()))).clone()
                })
            };
            if let Some(core) = core {
                let reply = core.call(op.clone()).await;
                if !matches!(reply, Reply::WrongGroup) { return reply; }
            }
            sleep(Duration::from_millis(40)).await;
            *self.config.lock().unwrap() = self.ctrl_ck.query().await;
        }
    }
    pub async fn get(&self, key: String) -> String { match self.call(Action::Get { key }).await { Reply::Get { value } => value.unwrap_or_default(), _ => unreachable!() } }
    pub async fn put(&self, key: String, value: String) { self.call(Action::Put { key, value }).await; }
    pub async fn append(&self, key: String, value: String) { self.call(Action::Append { key, value }).await; }
}
