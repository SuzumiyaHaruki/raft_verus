use super::msg::*;
use madsim::{net, time::*};
use std::{net::SocketAddr,sync::atomic::{AtomicU64,AtomicUsize,Ordering}};
pub(crate) static CLIENT_ID:AtomicU64=AtomicU64::new(1);
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub(crate) struct Command<T>{pub client:u64,pub seq:u64,pub args:T}
impl<T:net::rpc::Request+Clone> net::rpc::Request for Command<T>{type Response=T::Response;const ID:u64=T::ID;}

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
    id:u64,seq:AtomicU64,leader:AtomicUsize,serial:futures::lock::Mutex<()>,
    _mark: std::marker::PhantomData<(Req, Rsp)>,
}

impl<Req, Rsp> ClerkCore<Req, Rsp>
where
    Req: net::rpc::Request<Response = Result<Rsp, Error>> + Clone,
    Rsp: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(servers: Vec<SocketAddr>) -> Self {
        ClerkCore {
            servers,id:CLIENT_ID.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(1)).expect("client IDs exhausted"),
            seq:AtomicU64::new(0),leader:AtomicUsize::new(0),serial:futures::lock::Mutex::new(()),
            _mark: std::marker::PhantomData,
        }
    }

    pub async fn call(&self,args:Req)->Rsp {
        let _serial=self.serial.lock().await;
        assert!(!self.servers.is_empty(),"empty server set");
        let seq=self.seq.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(1)).expect("request IDs exhausted")+1;
        let cmd=Command{client:self.id,seq,args};
        let net=net::Endpoint::bind("0.0.0.0:0").await.expect("bind client RPC");
        let mut peer=self.leader.load(Ordering::SeqCst)%self.servers.len();
        loop {
            match net.call_timeout(self.servers[peer],cmd.clone(),Duration::from_millis(800)).await {
                Ok(Ok(value))=>{self.leader.store(peer,Ordering::SeqCst);return value;},
                _=>{peer=(peer+1)%self.servers.len();sleep(Duration::from_millis(10)).await;}
            }
        }
    }
}
