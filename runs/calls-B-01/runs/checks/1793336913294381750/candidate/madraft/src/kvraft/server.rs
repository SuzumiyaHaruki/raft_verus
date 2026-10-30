use super::msg::*;
use crate::raft;
use futures::StreamExt;
use madsim::{net, task, time::{sleep, Duration, Instant}};
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr, sync::{Arc, Mutex}, collections::HashMap};

pub trait State: Serialize + serde::de::DeserializeOwned + Send + Sync + Default + 'static {
    type Command: net::rpc::Request<Response = Result<Self::Output, Error>> + Clone;
    type Output: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;
    fn apply(&mut self, cmd: Self::Command) -> Self::Output;
}

#[derive(Serialize, Deserialize)]
struct Machine<S> {
    state: S,
    through: u64,
    sessions: HashMap<u64, (u64, Vec<u8>)>,
}

pub struct Server<S: State> {
    pub(crate) rf: raft::RaftHandle,
    me: usize,
    machine: Mutex<Machine<S>>,
    internal_client: u64,
    internal_sequence: futures::lock::Mutex<u64>,
}
impl<S: State> fmt::Debug for Server<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Server({})", self.me) }
}
impl<S: State> Server<S> {
    pub async fn new(servers: Vec<SocketAddr>, me: usize, max_raft_state: Option<usize>) -> Arc<Self> {
        let (rf, mut apply_ch) = raft::RaftHandle::new(servers, me).await;
        let this = Arc::new(Self { rf, me, internal_client: madsim::rand::random(), internal_sequence: futures::lock::Mutex::new(0), machine: Mutex::new(Machine { state: S::default(), through: 0, sessions: HashMap::new() }) });
        this.start_rpc_server();
        let worker = this.clone();
        task::spawn(async move {
            while let Some(msg) = apply_ch.next().await {
                match msg {
                    raft::ApplyMsg::Command { data, index } => {
                        let snapshot = {
                            let mut m = worker.machine.lock().unwrap();
                            if index <= m.through { continue; }
                            assert_eq!(index, m.through.checked_add(1).expect("application index exhausted"));
                            let request: Replicated = bincode::deserialize(&data).expect("replicated service command");
                            if super::sequence::new_request(m.sessions.get(&request.client).map(|(seq, _)| *seq), request.sequence) {
                                let cmd = bincode::deserialize(&request.command).expect("service command decode");
                                let result = m.state.apply(cmd);
                                m.sessions.insert(request.client, (request.sequence, bincode::serialize(&result).unwrap()));
                            }
                            m.through = index;
                            if max_raft_state.map_or(false, |limit| worker.rf.state_size() >= limit) {
                                Some(bincode::serialize(&*m).expect("service snapshot encode"))
                            } else { None }
                        };
                        if let Some(bytes) = snapshot { worker.rf.snapshot(index, &bytes).await.expect("service snapshot persist"); }
                    }
                    raft::ApplyMsg::Snapshot { data, term, index } => {
                        if worker.rf.cond_install_snapshot(term, index, &data).await {
                            let saved: Machine<S> = bincode::deserialize(&data).expect("service snapshot decode");
                            assert_eq!(saved.through, index);
                            let mut m = worker.machine.lock().unwrap();
                            if super::sequence::snapshot_boundary(saved.through, index, m.through) { *m = saved; }
                        }
                    }
                }
            }
        });
        this
    }
    pub(crate) fn inspect<R>(&self, f: impl FnOnce(&S) -> R) -> R { f(&self.machine.lock().unwrap().state) }
    pub(crate) async fn execute(&self, command: S::Command) -> Result<S::Output, Error> {
        let mut sequence = self.internal_sequence.lock().await;
        *sequence = super::sequence::next_sequence(*sequence).ok_or(Error::Failed)?;
        self.apply(ClientRequest { client: self.internal_client, sequence: *sequence, command }).await
    }
    fn start_rpc_server(self: &Arc<Self>) {
        let net = self.rf.endpoint.clone();
        let this = self.clone();
        net.add_rpc_handler(move |request: ClientRequest<S::Command>| {
            let this = this.clone();
            async move { this.apply(request).await }
        });
    }
    pub fn term(&self) -> u64 { self.rf.term() }
    pub fn is_leader(&self) -> bool { self.rf.is_leader() }
    async fn apply(&self, request: ClientRequest<S::Command>) -> Result<S::Output, Error> {
        if !self.rf.is_leader() { return Err(Error::NotLeader { hint: self.me }); }
        let encoded = Replicated { client: request.client, sequence: request.sequence,
            command: bincode::serialize(&request.command).map_err(|_| Error::Failed)? };
        let bytes = bincode::serialize(&encoded).map_err(|_| Error::Failed)?;
        let start = self.rf.start(&bytes).await.map_err(|_| Error::NotLeader { hint: self.me })?;
        let deadline = Instant::now() + Duration::from_millis(600);
        loop {
            {
                let m = self.machine.lock().unwrap();
                if let Some((seq, value)) = m.sessions.get(&request.client) {
                    if *seq == request.sequence { return bincode::deserialize(value).map_err(|_| Error::Failed); }
                    if *seq > request.sequence { return Err(Error::Failed); }
                }
            }
            if self.rf.term() != start.term || !self.rf.is_leader() { return Err(Error::NotLeader { hint: self.me }); }
            if Instant::now() >= deadline { return Err(Error::Timeout); }
            sleep(Duration::from_millis(5)).await;
        }
    }
}

pub type KvServer = Server<Kv>;
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Kv { values: HashMap<String, String> }
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
