use super::msg::*;
use crate::raft;
use madsim::net;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    collections::{HashMap, BTreeMap},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub trait State: Serialize + serde::de::DeserializeOwned + Send + Sync + Default + 'static {
    type Command: net::rpc::Request<Response = Result<Self::Output, Error>> + Clone;
    type Output: Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;
    fn apply(&mut self, cmd: Self::Command) -> Self::Output;
    fn retry_output(_output: &Self::Output) -> bool { false }
    // Services with their own migrated deduplication records need no second
    // durable copy of responses in this transport cache.
    const DURABLE_REPLIES: bool = true;
}

pub struct Server<S: State> {
    rf: raft::RaftHandle,
    me: usize,
    image: Mutex<Image<S>>,
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
            image: Mutex::new(Image { state: S::default(), replies: BTreeMap::new(), applied: 0 }),
        });
        let server = this.clone();
        madsim::task::spawn(async move {
            use futures::StreamExt;
            while let Some(message) = apply_ch.next().await {
                match message {
                    raft::ApplyMsg::Command { data, index } => {
                        let request: ClientRequest<S::Command> = bincode::deserialize(&data).expect("replicated request decode");
                        {
                            let mut image = server.image.lock().unwrap();
                            if index <= image.applied { continue; }
                            assert_eq!(index, image.applied + 1, "application gap");
                            let key = request.client;
                            if image.replies.get(&key).map_or(true, |(sequence, bytes)| *sequence < request.sequence ||
                                (*sequence == request.sequence && bincode::deserialize::<S::Output>(bytes).map_or(false, |o| S::retry_output(&o)))) {
                                let response = image.state.apply(request.command);
                                image.replies.insert(key, (request.sequence, bincode::serialize(&response).expect("response encode")));
                            }
                            image.applied = index;
                        }
                        if let Some(limit) = max_raft_state {
                            if madsim::fs::metadata("state").await.map_or(false, |m| m.len() as usize >= limit) {
                                let bytes = {
                                    let image = server.image.lock().unwrap();
                                    let empty = BTreeMap::new();
                                    bincode::serialize(&ImageRef { state: &image.state,
                                        replies: if S::DURABLE_REPLIES { &image.replies } else { &empty },
                                        applied: image.applied }).expect("snapshot encode")
                                };
                                server.rf.snapshot(index, &bytes).await.expect("snapshot persistence");
                            }
                        }
                    }
                    raft::ApplyMsg::Snapshot { data, term, index } => {
                        if server.rf.cond_install_snapshot(term, index, &data).await {
                            let restored: Image<S> = bincode::deserialize(&data).expect("service snapshot decode");
                            assert_eq!(restored.applied, index);
                            let mut image = server.image.lock().unwrap();
                            if index > image.applied { *image = restored; }
                        }
                    }
                }
            }
        });
        this.start_rpc_server();
        this
    }

    pub(crate) fn inspect<T>(&self, f: impl FnOnce(&S) -> T) -> T { f(&self.image.lock().unwrap().state) }
    pub(crate) fn endpoint(&self) -> net::Endpoint { self.rf.endpoint.clone() }

    fn start_rpc_server(self: &Arc<Self>) {
        let net = self.rf.endpoint.clone();

        let this = self.clone();
        net.add_rpc_handler(move |cmd: ClientRequest<S::Command>| {
            let this = this.clone();
            async move { this.apply(cmd).await }
        });
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.rf.term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.rf.is_leader()
    }

    pub(crate) async fn apply(&self, cmd: ClientRequest<S::Command>) -> Result<S::Output, Error> {
        let key = cmd.client;
        let sequence = cmd.sequence;
        if !self.rf.is_leader() { return Err(Error::NotLeader { hint: self.me }); }
        let bytes = bincode::serialize(&cmd).map_err(|_| Error::Failed)?;
        let start = self.rf.start(&bytes).await.map_err(|e| match e {
            raft::Error::NotLeader(hint) => Error::NotLeader { hint }, _ => Error::Failed,
        })?;
        let deadline = madsim::time::Instant::now() + madsim::time::Duration::from_millis(600);
        loop {
            if let Some((last, bytes)) = self.image.lock().unwrap().replies.get(&key) {
                if *last == sequence { return bincode::deserialize(bytes).map_err(|_| Error::Failed); }
                if *last > sequence { return Err(Error::Failed); }
            }
            if self.rf.term() != start.term || !self.rf.is_leader() { return Err(Error::Failed); }
            if madsim::time::Instant::now() >= deadline { return Err(Error::Timeout); }
            madsim::time::sleep(madsim::time::Duration::from_millis(10)).await;
        }
    }
}

#[derive(Serialize)]
struct ImageRef<'a, S> {
    state: &'a S,
    replies: &'a BTreeMap<u128, (u64, Vec<u8>)>,
    applied: u64,
}
#[derive(Serialize, Deserialize)]
struct Image<S> {
    state: S,
    replies: BTreeMap<u128, (u64, Vec<u8>)>,
    applied: u64,
}

pub type KvServer = Server<Kv>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Kv {
    values: HashMap<String, String>,
}

impl State for Kv {
    type Command = Op;
    type Output = String;

    fn apply(&mut self, cmd: Self::Command) -> Self::Output {
        match cmd {
            Op::Get { key } => self.values.get(&key).cloned().unwrap_or_default(),
            Op::Put { key, value } => { self.values.insert(key, value); String::new() },
            Op::Append { key, value } => { self.values.entry(key).or_default().push_str(&value); String::new() },
        }
    }
}
