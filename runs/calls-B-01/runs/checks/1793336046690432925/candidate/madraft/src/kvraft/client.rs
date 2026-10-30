use super::msg::*;
use madsim::{net, time::*};
use std::{net::SocketAddr, sync::atomic::{AtomicUsize, Ordering}};
use madsim::rand::Rng;

pub struct Clerk {
    core: ClerkCore<Op, String>,
}

impl Clerk {
    pub fn new(servers: Vec<SocketAddr>) -> Clerk {
        Clerk {
            core: ClerkCore::new(servers),
        }
    }

    /// fetch the current value for a key.
    /// returns "" if the key does not exist.
    /// keeps trying forever in the face of all other errors.
    pub async fn get(&self, key: String) -> String {
        self.core.call(Op::Get { key }).await
    }

    pub async fn put(&self, key: String, value: String) {
        self.core.call(Op::Put { key, value }).await;
    }

    pub async fn append(&self, key: String, value: String) {
        self.core.call(Op::Append { key, value }).await;
    }
}

pub struct ClerkCore<Req, Rsp> {
    servers: Vec<SocketAddr>,
    _mark: std::marker::PhantomData<(Req, Rsp)>,
    client: u64,
    sequence: futures::lock::Mutex<u64>,
    leader: AtomicUsize,
}

impl<Req, Rsp> ClerkCore<Req, Rsp>
where
    Req: net::rpc::Request<Response = Result<Rsp, Error>> + Clone,
    Rsp: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        // Retained controller clients execute on MadSim's main node. The
        // pinned simulator leaves it without an IP and silently drops sends.
        // Give that host a stable source address; do not alter peer topology.
        #[cfg(madsim)]
        if madsim::plugin::node().to_string() == "0" {
            net::NetSim::current().set_ip(madsim::plugin::node(), std::net::Ipv4Addr::new(192, 0, 2, 254).into());
        }
        ClerkCore {
            servers,
            client: madsim::rand::thread_rng().gen(),
            sequence: futures::lock::Mutex::new(0),
            leader: AtomicUsize::new(0),
            _mark: std::marker::PhantomData,
        }
    }

    pub async fn call(&self, args: Req) -> Rsp {
        assert!(!self.servers.is_empty(), "empty service membership");
        let mut sequence = self.sequence.lock().await;
        *sequence = super::sequence::next_sequence(*sequence).expect("client sequence exhausted");
        let args = ClientRequest { client: self.client, sequence: *sequence, command: args };
        let net = net::Endpoint::bind("0.0.0.0:0").await.expect("bind client RPC");
        let mut peer = self.leader.load(Ordering::Relaxed) % self.servers.len();
        loop {
            match net.call_timeout(self.servers[peer], args.clone(), Duration::from_millis(700)).await {
                Ok(Ok(value)) => { self.leader.store(peer, Ordering::Relaxed); return value; }
                Ok(Err(Error::NotLeader { hint })) if hint < self.servers.len() && hint != peer => peer = hint,
                _ => peer = (peer + 1) % self.servers.len(),
            }
            sleep(Duration::from_millis(15)).await;
        }
    }
}
