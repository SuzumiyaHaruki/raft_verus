use super::msg::*;
use crate::raft;
use madsim::{net,task,time::{timeout,Duration}};
use futures::{StreamExt,channel::oneshot};
use super::client::Command;
use std::{collections::HashMap,sync::Mutex};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    net::SocketAddr,
    sync::Arc,
};

pub trait State: Serialize + serde::de::DeserializeOwned + Send + Sync + Default + 'static {
    type Command: net::rpc::Request<Response = Result<Self::Output, Error>> + Clone;
    type Output: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;
    fn apply(&mut self, cmd: Self::Command) -> Self::Output;
    fn apply_request(&mut self,client:u64,seq:u64,cmd:Self::Command)->Self::Output {let _=(client,seq);self.apply(cmd)}
    fn cache_result(_value:&Self::Output)->bool {true}
    fn use_sessions()->bool {true}
}

#[derive(Serialize,Deserialize)]
struct ServiceImage<S>{state:S,sessions:HashMap<u64,(u64,Vec<u8>)>,index:u64}
struct Inner<S>{image:ServiceImage<S>,pending:HashMap<(u64,u64),Vec<oneshot::Sender<Vec<u8>>>>}
pub struct Server<S: State> {
    rf: raft::RaftHandle,
    me: usize,
    inner:Mutex<Inner<S>>,
}

impl<S: State> fmt::Debug for Server<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server({})", self.me)
    }
}

impl<S: State> Server<S> {
    pub async fn new(
        servers: Vec<SocketAddr>,
        me: usize,
        max_raft_state: Option<usize>,
    ) -> Arc<Self> {
        // You may need initialization code here.
        let (rf, mut apply_ch) = raft::RaftHandle::new(servers, me).await;

        let this = Arc::new(Server {
            rf,
            me,
            inner:Mutex::new(Inner{image:ServiceImage{state:S::default(),sessions:HashMap::new(),index:0},pending:HashMap::new()}),
        });
        let owner=this.clone();
        task::spawn(async move {
            while let Some(msg)=apply_ch.next().await {
                let snapshot=match msg {
                    raft::ApplyMsg::Command{data,index}=>{
                        let cmd:Command<S::Command>=bincode::deserialize(&data).expect("invalid replicated command");
                        let mut inner=owner.inner.lock().unwrap();
                        if index<=inner.image.index {continue;}
                        assert_eq!(index,inner.image.index+1,"application gap");
                        let value=if let Some((seq,value))=inner.image.sessions.get(&cmd.client) {
                            if S::use_sessions() && *seq>=cmd.seq {Some(value.clone())} else {None}
                        } else {None};
                        let value=value.unwrap_or_else(||bincode::serialize(&inner.image.state.apply_request(cmd.client,cmd.seq,cmd.args.clone())).unwrap());
                        let old=inner.image.sessions.get(&cmd.client).map(|v|v.0).unwrap_or(0);
                        if S::use_sessions() && cmd.seq>old && S::cache_result(&bincode::deserialize(&value).unwrap()) {inner.image.sessions.insert(cmd.client,(cmd.seq,value.clone()));}
                        inner.image.index=index;
                        if let Some(waiters)=inner.pending.remove(&(cmd.client,cmd.seq)) {
                            for waiter in waiters {let _=waiter.send(value.clone());}
                        }
                        Some((index,bincode::serialize(&inner.image).unwrap()))
                    },
                    raft::ApplyMsg::Snapshot{data,term,index}=>{
                        if owner.rf.cond_install_snapshot(term,index,&data).await {
                            let image:ServiceImage<S>=bincode::deserialize(&data).expect("invalid service snapshot");
                            assert_eq!(image.index,index,"snapshot boundary mismatch");
                            let mut inner=owner.inner.lock().unwrap();
                            if index>inner.image.index {inner.image=image;}
                        }
                        None
                    }
                };
                if let (Some(limit),Some((index,bytes)))=(max_raft_state,snapshot) {
                    if owner.rf.state_size().await>=limit {owner.rf.snapshot(index,&bytes).await.expect("snapshot storage");}
                }
            }
        });
        this.start_rpc_server();
        this
    }

    fn start_rpc_server(self: &Arc<Self>) {
        let net = self.rf.endpoint.clone();

        let this = self.clone();
        net.add_rpc_handler(move |cmd: Command<S::Command>| {
            let this = this.clone();
            async move { this.apply(cmd).await }
        });
    }

    pub(crate) fn inspect<R>(&self,f:impl FnOnce(&S)->R)->R {f(&self.inner.lock().unwrap().image.state)}
    pub(crate) fn initialize(&self,f:impl FnOnce(&mut S)) {f(&mut self.inner.lock().unwrap().image.state)}
    pub(crate) fn endpoint(&self)->net::Endpoint {self.rf.endpoint.clone()}
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.rf.term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.rf.is_leader()
    }

    pub(crate) async fn apply(&self,cmd:Command<S::Command>)->Result<S::Output,Error> {
        if !self.rf.is_leader() {return Err(Error::NotLeader{hint:(self.me+1)});}
        let key=(cmd.client,cmd.seq);
        let (send,recv)=oneshot::channel();
        {
            let mut inner=self.inner.lock().unwrap();
            if let Some((seq,value))=inner.image.sessions.get(&cmd.client) {
                if S::use_sessions() && *seq==cmd.seq {return Ok(bincode::deserialize(value).unwrap());}
            }
            inner.pending.entry(key).or_default().push(send);
        }
        let bytes=bincode::serialize(&cmd).map_err(|_|Error::Failed)?;
        if self.rf.start(&bytes).await.is_err() {self.inner.lock().unwrap().pending.remove(&key);return Err(Error::Failed);}
        match timeout(Duration::from_millis(700),recv).await {
            Ok(Ok(value))=>bincode::deserialize(&value).map_err(|_|Error::Failed),
            _=>{self.inner.lock().unwrap().pending.remove(&key);Err(Error::Timeout)}
        }
    }

}

pub type KvServer = Server<Kv>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Kv {
    data:HashMap<String,String>,
}

impl State for Kv {
    type Command = Op;
    type Output = String;

    fn apply(&mut self, cmd: Self::Command) -> Self::Output {
        match cmd {
            Op::Get{key}=>self.data.get(&key).cloned().unwrap_or_default(),
            Op::Put{key,value}=>{self.data.insert(key,value);String::new()},
            Op::Append{key,value}=>{self.data.entry(key).or_default().push_str(&value);String::new()},
        }
    }
}
