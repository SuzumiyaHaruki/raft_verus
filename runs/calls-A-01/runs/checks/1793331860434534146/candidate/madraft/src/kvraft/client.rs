use super::msg::*;
use madsim::{net, time::*};
use std::net::SocketAddr;

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
    client: u128,
    sequence: std::sync::atomic::AtomicU64,
    _mark: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp> ClerkCore<Req, Rsp>
where
    Req: net::rpc::Request<Response = Result<Rsp, Error>> + Clone,
    Rsp: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        ClerkCore {
            servers,
            client: madsim::rand::random(),
            sequence: std::sync::atomic::AtomicU64::new(0),
            _mark: std::marker::PhantomData,
        }
    }

    pub async fn call(&self, args: Req) -> Rsp {
        let net = net::Endpoint::bind("0.0.0.0:0").await.expect("bind client RPC");
        let sequence = self.sequence.fetch_update(std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst, |s| s.checked_add(1)).expect("request sequence exhausted");
        let request = ClientRequest { client: self.client, sequence, command: args };
        loop {
            for addr in &self.servers {
                if let Ok(Ok(response)) = net.call_timeout::<ClientRequest<Req>>(
                    *addr, request.clone(), Duration::from_millis(700)).await {
                    return response;
                }
            }
            sleep(Duration::from_millis(30)).await;
        }
    }
}
