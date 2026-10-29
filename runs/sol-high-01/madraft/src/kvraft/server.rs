use super::{client::Request, msg::*};
use crate::raft;
use futures::{lock::Mutex as AsyncMutex, StreamExt};
use madsim::{net, rand::{self, Rng}, task, time::*};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, net::SocketAddr, sync::{Arc, Mutex}};

pub trait State: Serialize + serde::de::DeserializeOwned + Send + Sync + Default + 'static {
    type Command: net::rpc::Request<Response = Result<Self::Output, Error>> + Clone;
    type Output: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;
    fn apply(&mut self, cmd: Self::Command) -> Self::Output;
    fn apply_request(&mut self, _client: u128, _sequence: u64, cmd: Self::Command) -> Self::Output { self.apply(cmd) }
    fn deduplicate() -> bool { true }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Service<S> {
    pub state: S,
    sessions: BTreeMap<u128, (u64, Vec<u8>)>,
    applied: u64,
    #[serde(skip)]
    recent: BTreeMap<u64, (u128, u64, Vec<u8>)>,
}

pub struct Server<S: State> {
    pub(crate) rf: raft::RaftHandle,
    me: usize,
    pub(crate) service: Mutex<Service<S>>,
    internal_id: u128,
    internal_sequence: AsyncMutex<u64>,
}
impl<S: State> fmt::Debug for Server<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Server({})", self.me) }
}
impl<S: State> Server<S> {
    pub async fn new(servers: Vec<SocketAddr>, me: usize, max_raft_state: Option<usize>) -> Arc<Self> {
        Self::new_with_state(servers, me, max_raft_state, S::default()).await
    }
    pub(crate) async fn new_with_state(servers: Vec<SocketAddr>, me: usize, max_raft_state: Option<usize>, state: S) -> Arc<Self> {
        let (rf, mut apply_ch) = raft::RaftHandle::new(servers, me).await;
        let this = Arc::new(Self { rf, me,
            service: Mutex::new(Service { state, sessions: BTreeMap::new(), applied: 0, recent: BTreeMap::new() }),
            internal_id: rand::thread_rng().gen(), internal_sequence: AsyncMutex::new(0) });
        let copy = this.clone();
        task::spawn(async move {
            while let Some(msg) = apply_ch.next().await {
                match msg {
                    raft::ApplyMsg::Command { data, index } => {
                        let request: Request<S::Command> = bincode::deserialize(&data).expect("decode committed service command");
                        let snapshot = {
                            let mut service = copy.service.lock().unwrap();
                            if index <= service.applied { continue; }
                            assert_eq!(index, service.applied + 1, "service application out of order");
                            let output = if S::deduplicate() && service.sessions.get(&request.client).map_or(false, |r| r.0 >= request.sequence) {
                                service.sessions.get(&request.client).filter(|r| r.0 == request.sequence).map(|r| r.1.clone())
                            } else {
                                let output = service.state.apply_request(request.client, request.sequence, request.command);
                                let output = bincode::serialize(&output).expect("encode service output");
                                if S::deduplicate() { service.sessions.insert(request.client, (request.sequence, output.clone())); }
                                Some(output)
                            };
                            if let Some(output) = output {
                                service.recent.insert(index, (request.client, request.sequence, output));
                                while service.recent.len() > 128 { let first = *service.recent.keys().next().unwrap(); service.recent.remove(&first); }
                            }
                            service.applied = index;
                            if max_raft_state.map_or(false, |limit| copy.rf.state_size() >= limit / 2) {
                                Some(bincode::serialize(&*service).expect("encode service snapshot"))
                            } else { None }
                        };
                        if let Some(snapshot) = snapshot { copy.rf.snapshot(index, &snapshot).await.expect("persist service snapshot"); }
                    }
                    raft::ApplyMsg::Snapshot { data, term, index } => {
                        if copy.rf.cond_install_snapshot(term, index, &data).await {
                            let restored: Service<S> = bincode::deserialize(&data).expect("decode service snapshot");
                            assert_eq!(restored.applied, index, "snapshot cursor mismatch");
                            let mut service = copy.service.lock().unwrap();
                            if service.applied < index { *service = restored; }
                        }
                    }
                }
            }
        });
        this.start_rpc_server();
        this
    }
    fn start_rpc_server(self: &Arc<Self>) {
        let this = self.clone();
        self.rf.endpoint.add_rpc_handler(move |cmd: Request<S::Command>| {
            let this = this.clone(); async move { this.apply(cmd).await }
        });
    }
    pub fn term(&self) -> u64 { self.rf.term() }
    pub fn is_leader(&self) -> bool { self.rf.is_leader() }
    pub(crate) async fn submit(&self, command: S::Command) -> Result<S::Output, Error> {
        let mut sequence = self.internal_sequence.lock().await;
        *sequence = sequence.checked_add(1).ok_or(Error::Failed)?;
        self.apply(Request { client: self.internal_id, sequence: *sequence, command }).await
    }
    async fn apply(&self, cmd: Request<S::Command>) -> Result<S::Output, Error> {
        let bytes = bincode::serialize(&cmd).map_err(|_| Error::Failed)?;
        let start = self.rf.start(&bytes).await.map_err(|e| match e {
            raft::Error::NotLeader(hint) => Error::NotLeader { hint }, _ => Error::Failed,
        })?;
        let deadline = Instant::now() + Duration::from_millis(550);
        loop {
            {
                let service = self.service.lock().unwrap();
                if let Some((sequence, output)) = service.sessions.get(&cmd.client) {
                    if *sequence == cmd.sequence { return bincode::deserialize(output).map_err(|_| Error::Failed); }
                    if *sequence > cmd.sequence { return Err(Error::Failed); }
                }
                if let Some((client, sequence, output)) = service.recent.get(&start.index) {
                    if *client == cmd.client && *sequence == cmd.sequence { return bincode::deserialize(output).map_err(|_| Error::Failed); }
                }
                if service.applied >= start.index { return Err(Error::Failed); }
            }
            if self.rf.term() != start.term || !self.rf.is_leader() { return Err(Error::NotLeader { hint: self.rf.leader_hint() }); }
            if Instant::now() >= deadline { return Err(Error::Timeout); }
            sleep(Duration::from_millis(5)).await;
        }
    }
}
pub type KvServer = Server<Kv>;
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Kv { values: BTreeMap<String, String> }
impl State for Kv {
    type Command = Op;
    type Output = String;
    fn apply(&mut self, cmd: Op) -> String {
        match cmd {
            Op::Get { key } => self.values.get(&key).cloned().unwrap_or_default(),
            Op::Put { key, value } => { self.values.insert(key, value); String::new() }
            Op::Append { key, value } => { self.values.entry(key).or_default().push_str(&value); String::new() }
        }
    }
}
