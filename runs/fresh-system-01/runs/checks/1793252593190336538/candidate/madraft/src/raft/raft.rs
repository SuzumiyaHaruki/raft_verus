use futures::channel::mpsc;
use madsim::{
    fs, net,
    rand::{self, Rng},
    task,
    time::*,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct RaftHandle {
    pub(crate) endpoint: net::Endpoint,
    inner: Arc<Mutex<Raft>>,
    serial: Arc<futures::lock::Mutex<()>>,
}

type MsgSender = mpsc::UnboundedSender<ApplyMsg>;
pub type MsgRecver = mpsc::UnboundedReceiver<ApplyMsg>;

/// As each Raft peer becomes aware that successive log entries are committed,
/// the peer should send an `ApplyMsg` to the service (or tester) on the same
/// server, via the `apply_ch` passed to `Raft::new`.
pub enum ApplyMsg {
    Command {
        data: Vec<u8>,
        index: u64,
    },
    // For 2D:
    Snapshot {
        data: Vec<u8>,
        term: u64,
        index: u64,
    },
}

#[derive(Debug)]
pub struct Start {
    /// The index that the command will appear at if it's ever committed.
    pub index: u64,
    /// The current term.
    pub term: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("this node is not a leader, next leader: {0}")]
    NotLeader(usize),
    #[error("IO error")]
    IO(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[path = "election.rs"]
mod election;
use election::Election;

struct Raft {
    peers: Vec<SocketAddr>,
    me: usize,
    apply_ch: MsgSender,
    state: Election,
    deadline: Instant,
    heartbeat_due: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Persist {
    term: u64,
    voted_for: Option<usize>,
}

impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Raft({})", self.me) }
}

impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        assert!(!peers.is_empty() && me < peers.len(), "invalid membership");
        assert!((0..peers.len()).all(|i| (0..i).all(|j| peers[i] != peers[j])), "duplicate peer");
        let endpoint = net::Endpoint::bind(peers[me]).await.expect("bind Raft RPC");
        let (apply_ch, recver) = mpsc::unbounded();
        let state = Election::new(peers.len(), me).expect("valid membership");
        let inner = Arc::new(Mutex::new(Raft {
            peers, me, apply_ch, state,
            deadline: Instant::now() + Raft::election_timeout(),
            heartbeat_due: Instant::now(),
        }));
        let handle = Self { inner, endpoint, serial: Arc::new(futures::lock::Mutex::new(())) };
        handle.restore().await.expect("invalid or unavailable durable image: node halted");
        handle.start_rpc_server();
        handle.start_timer();
        (handle, recver)
    }

    pub async fn start(&self, _cmd: &[u8]) -> Result<Start> {
        let raft = self.inner.lock().unwrap();
        if !raft.state.is_leader() { return Err(Error::NotLeader((raft.me + 1) % raft.peers.len())); }
        todo!("2B proposals")
    }
    pub fn term(&self) -> u64 { self.inner.lock().unwrap().state.term }
    pub fn is_leader(&self) -> bool { self.inner.lock().unwrap().state.is_leader() }
    pub async fn cond_install_snapshot(&self, _term: u64, _index: u64, _snapshot: &[u8]) -> bool {
        todo!("2D snapshot installation")
    }
    pub async fn snapshot(&self, _index: u64, _snapshot: &[u8]) -> Result<()> {
        todo!("2D snapshot creation")
    }

    // The serial guard covers mutation, durable completion, and reply selection.
    // An incomplete/truncated existing image is a fail-stop condition on restore;
    // it is never interpreted as permission to discard an unknown prior vote.
    async fn persist(&self) -> io::Result<()> {
        let bytes = {
            let raft = self.inner.lock().unwrap();
            bincode::serialize(&Persist { term: raft.state.term, voted_for: raft.state.voted_for })
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };
        let file = fs::File::create("state").await?;
        file.write_all_at(&bytes, 0).await?;
        file.sync_all().await?;
        Ok(())
    }
    async fn restore(&self) -> io::Result<()> {
        match fs::read("state").await {
            Ok(bytes) => {
                let image: Persist = bincode::deserialize(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let mut raft = self.inner.lock().unwrap();
                if image.voted_for.map_or(false, |v| v >= raft.peers.len()) {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid durable voter"));
                }
                raft.state.term = image.term;
                raft.state.voted_for = image.voted_for;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => { self.persist().await?; }
            Err(e) => return Err(e),
        }
        Ok(())
    }
    fn start_rpc_server(&self) {
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: RequestVoteArgs| {
            let this = this.clone();
            async move { this.request_vote(args).await.expect("vote storage failed: node halted") }
        });
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: HeartbeatArgs| {
            let this = this.clone();
            async move { this.heartbeat(args).await.expect("heartbeat storage failed: node halted") }
        });
    }
    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        let _serial = self.serial.lock().await;
        let reply = {
            let mut raft = self.inner.lock().unwrap();
            let granted = raft.state.request_vote(args.term, args.candidate, args.last_term, args.last_index);
            if granted { raft.reset_deadline(); }
            RequestVoteReply { term: raft.state.term, granted }
        };
        self.persist().await?;
        Ok(reply)
    }
    async fn heartbeat(&self, args: HeartbeatArgs) -> Result<RequestVoteReply> {
        let _serial = self.serial.lock().await;
        let reply = {
            let mut raft = self.inner.lock().unwrap();
            let accepted = raft.state.heartbeat(args.term, args.leader);
            if accepted { raft.reset_deadline(); }
            RequestVoteReply { term: raft.state.term, granted: accepted }
        };
        self.persist().await?;
        Ok(reply)
    }
    fn start_timer(&self) {
        let this = self.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(10)).await;
                this.tick().await;
            }
        });
    }
    async fn tick(&self) {
        let _serial = self.serial.lock().await;
        let (votes, heartbeat) = {
            let mut raft = self.inner.lock().unwrap();
            let now = Instant::now();
            if raft.state.is_leader() {
                if now < raft.heartbeat_due { return; }
                raft.heartbeat_due = now + Duration::from_millis(80);
                (None, Some(HeartbeatArgs { term: raft.state.term, leader: raft.me }))
            } else {
                if now < raft.deadline { return; }
                raft.reset_deadline();
                if !raft.state.campaign() { return; }
                let args = RequestVoteArgs { term: raft.state.term, candidate: raft.me,
                    last_term: raft.state.last_term, last_index: raft.state.last_index };
                // The self-vote also handles a singleton group.
                let me = raft.me;
                raft.state.receive_grant(args.term, me, true);
                (Some(args), None)
            }
        };
        self.persist().await.expect("timer storage failed: node halted");
        let peers = self.inner.lock().unwrap().peers.clone();
        let me = self.inner.lock().unwrap().me;
        for (peer, addr) in peers.into_iter().enumerate() {
            if peer == me { continue; }
            let this = self.clone();
            if let Some(args) = votes.clone() {
                task::spawn(async move {
                    if let Ok(reply) = this.endpoint.call_timeout(addr, args, Duration::from_millis(250)).await {
                        this.vote_reply(peer, reply).await;
                    }
                });
            } else if let Some(args) = heartbeat.clone() {
                task::spawn(async move {
                    if let Ok(reply) = this.endpoint.call_timeout(addr, args, Duration::from_millis(250)).await {
                        this.term_reply(peer, reply).await;
                    }
                });
            }
        }
    }
    async fn vote_reply(&self, peer: usize, reply: RequestVoteReply) {
        let _serial = self.serial.lock().await;
        {
            let mut raft = self.inner.lock().unwrap();
            if raft.state.receive_grant(reply.term, peer, reply.granted) {
                raft.heartbeat_due = Instant::now();
            }
        }
        self.persist().await.expect("reply storage failed: node halted");
    }
    async fn term_reply(&self, peer: usize, reply: RequestVoteReply) {
        let _serial = self.serial.lock().await;
        {
            let mut raft = self.inner.lock().unwrap();
            if peer < raft.peers.len() { raft.state.observe_term(reply.term); }
        }
        self.persist().await.expect("reply storage failed: node halted");
    }
}
impl Raft {
    fn election_timeout() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(350..650))
    }
    fn reset_deadline(&mut self) { self.deadline = Instant::now() + Self::election_timeout(); }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteArgs { term: u64, candidate: usize, last_term: u64, last_index: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteReply { term: u64, granted: bool }
impl net::rpc::Request for RequestVoteArgs {
    type Response = RequestVoteReply;
    const ID: u64 = 1;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatArgs { term: u64, leader: usize }
impl net::rpc::Request for HeartbeatArgs {
    type Response = RequestVoteReply;
    const ID: u64 = 10;
}
