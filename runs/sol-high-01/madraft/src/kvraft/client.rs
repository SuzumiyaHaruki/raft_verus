use super::msg::*;
use madsim::{net, rand::{self, Rng}, time::*};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::atomic::{AtomicUsize, Ordering}};
use futures::lock::Mutex;

pub struct Clerk { core: ClerkCore<Op, String> }
impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Self { Self { core: ClerkCore::new(servers) } }
    pub async fn get(&self, key: String) -> String { self.core.call(Op::Get { key }).await }
    pub async fn put(&self, key: String, value: String) { self.core.call(Op::Put { key, value }).await; }
    pub async fn append(&self, key: String, value: String) { self.core.call(Op::Append { key, value }).await; }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request<C> { pub client: u128, pub sequence: u64, pub command: C }
impl<C, R> net::rpc::Request for Request<C>
where C: net::rpc::Request<Response = Result<R, Error>> + Clone,
    R: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    type Response = Result<R, Error>;
    const ID: u64 = C::ID + 100;
}

pub struct ClerkCore<Req, Rsp> {
    servers: Vec<SocketAddr>,
    client: u128,
    sequence: Mutex<u64>,
    leader: AtomicUsize,
    endpoint: std::sync::Mutex<Option<net::Endpoint>>,
    _mark: std::marker::PhantomData<(Req, Rsp)>,
}
impl<Req, Rsp> ClerkCore<Req, Rsp>
where Req: net::rpc::Request<Response = Result<Rsp, Error>> + Clone,
    Rsp: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        configure_client_host();
        Self { servers, client: rand::thread_rng().gen(), sequence: Mutex::new(0),
            leader: AtomicUsize::new(0), endpoint: std::sync::Mutex::new(None), _mark: std::marker::PhantomData }
    }
    pub(crate) fn use_endpoint(&self, endpoint: net::Endpoint) { *self.endpoint.lock().unwrap() = Some(endpoint); }
    pub async fn call(&self, command: Req) -> Rsp {
        // One outstanding request per session permits bounded deduplication.
        let mut sequence = self.sequence.lock().await;
        *sequence = sequence.checked_add(1).expect("client sequence exhausted");
        let args = Request { client: self.client, sequence: *sequence, command };
        let existing = self.endpoint.lock().unwrap().clone();
        let net = match existing { Some(endpoint) => endpoint,
            None => net::Endpoint::bind("0.0.0.0:0").await.expect("bind client RPC") };
        assert!(!self.servers.is_empty(), "empty service membership");
        let mut peer = self.leader.load(Ordering::Relaxed) % self.servers.len();
        loop {
            match net.call_timeout(self.servers[peer], args.clone(), Duration::from_millis(650)).await {
                Ok(Ok(result)) => { self.leader.store(peer, Ordering::Relaxed); return result; }
                _ => peer = (peer + 1) % self.servers.len(),
            }
            sleep(Duration::from_millis(20)).await;
        }
    }
}

// MadSim 0.2 leaves its main task without an IP. The retained controller and
// shard clients run on that task, unlike KV clients on explicitly named nodes.
// Supply only that host's deployment address; never move traffic off its node.
pub(crate) fn configure_client_host() {
    #[cfg(madsim)]
    if madsim::plugin::node().to_string() == "0" {
        madsim::net::NetSim::current().set_ip(madsim::plugin::node(), "10.255.255.254".parse().unwrap());
    }
}
