use crate::{shard_ctrler::client::Clerk as CtrlerClerk, kvraft::msg::ClientRequest};
use super::msg::*;
use madsim::{net, time::*};
use std::{net::SocketAddr, sync::atomic::{AtomicU64, Ordering}};
pub struct Clerk {
    ctrl_ck: CtrlerClerk, client: u128, sequence: AtomicU64,
    gate: futures::lock::Mutex<()>,
}
impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        Self { ctrl_ck: CtrlerClerk::new(servers), client: madsim::rand::random(), sequence: AtomicU64::new(0), gate: futures::lock::Mutex::new(()) }
    }
    async fn call(&self, key: &str, operation: Op) -> Reply {
        let _gate = self.gate.lock().await;
        crate::kvraft::client::prepare_client_host();
        let net = net::Endpoint::bind("0.0.0.0:0").await.expect("shard client bind");
        let sequence = self.sequence.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |s| s.checked_add(1)).expect("sequence exhausted");
        loop {
            let config = self.ctrl_ck.query().await;
            let gid = config.shards[super::key2shard(key)];
            if let Some(peers) = config.groups.get(&gid) {
                let request = ClientRequest { client: self.client, sequence, command: Op::Client {
                    client: self.client, sequence, config: config.num, operation: Box::new(operation.clone()),
                }};
                for peer in peers {
                    if let Ok(Ok(reply)) = net.call_timeout(*peer, request.clone(), Duration::from_millis(700)).await {
                        if !matches!(reply, Reply::WrongGroup) { return reply; }
                    }
                }
            }
            sleep(Duration::from_millis(60)).await;
        }
    }
    pub async fn get(&self, key: String) -> String {
        match self.call(&key, Op::Get { key: key.clone() }).await {
            Reply::Get { value } => value.unwrap_or_default(), _ => unreachable!(),
        }
    }
    pub async fn put(&self, key: String, value: String) { self.call(&key, Op::Put { key: key.clone(), value }).await; }
    pub async fn append(&self, key: String, value: String) { self.call(&key, Op::Append { key: key.clone(), value }).await; }
}
