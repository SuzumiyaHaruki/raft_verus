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

struct Raft {
    endpoint: net::Endpoint,
    peers: Vec<SocketAddr>,
    me: usize,
    apply_ch: MsgSender,

    // Your data here (2A, 2B, 2C).
    // Look at the paper's Figure 2 for a description of what
    // state a Raft server must maintain.
    state: Election,
    voted: Vec<bool>,
    deadline: Instant,
    journal_len: u64,
    leader: usize,
}

#[path = "election.rs"]
mod election;
use election::{Election, Role, Vote};

impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Raft({})", self.me)
    }
}

// HINT: put async functions here
impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        let endpoint = net::Endpoint::bind(peers[me]).await.expect("bind Raft RPC");
        let (apply_ch, recver) = mpsc::unbounded();
        let inner = Arc::new(Mutex::new(Raft {
            endpoint: endpoint.clone(),
            peers: peers.clone(),
            me,
            apply_ch,
            state: election::new(),
            voted: vec![false; peers.len()],
            deadline: Instant::now() + Raft::generate_election_timeout(),
            journal_len: 0,
            leader: me,
        }));
        let handle = RaftHandle { inner, endpoint, serial: Arc::new(futures::lock::Mutex::new(())) };
        // initialize from state persisted before a crash
        handle.restore().await.expect("failed to restore");
        handle.start_rpc_server();
        handle.start_timer();

        (handle, recver)
    }

    /// Start agreement on the next command to be appended to Raft's log.
    ///
    /// If this server isn't the leader, returns [`Error::NotLeader`].
    /// Otherwise start the agreement and return immediately.
    ///
    /// There is no guarantee that this command will ever be committed to the
    /// Raft log, since the leader may fail or lose an election.
    pub async fn start(&self, cmd: &[u8]) -> Result<Start> {
        let mut raft = self.inner.lock().unwrap();
        info!("{:?} start", *raft);
        raft.start(cmd)
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        let raft = self.inner.lock().unwrap();
        raft.state.term
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        let raft = self.inner.lock().unwrap();
        matches!(raft.state.role, Role::Leader)
    }

    /// A service wants to switch to snapshot.  
    ///
    /// Only do so if Raft hasn't have more recent info since it communicate
    /// the snapshot on `apply_ch`.
    pub async fn cond_install_snapshot(
        &self,
        last_included_term: u64,
        last_included_index: u64,
        snapshot: &[u8],
    ) -> bool {
        todo!()
    }

    /// The service says it has created a snapshot that has all info up to and
    /// including index. This means the service no longer needs the log through
    /// (and including) that index. Raft should now trim its log as much as
    /// possible.
    pub async fn snapshot(&self, index: u64, snapshot: &[u8]) -> Result<()> {
        todo!()
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    async fn persist(&self) -> io::Result<()> {
        // Two self-contained generations. Creating the inactive file may
        // truncate it; the previous complete generation remains recoverable.
        let (bytes, generation) = {
            let r = self.inner.lock().unwrap();
            let generation = r.journal_len.checked_add(1)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "generation exhausted"))?;
            let mut bytes = Vec::with_capacity(25);
            bytes.extend_from_slice(&generation.to_le_bytes());
            bytes.extend_from_slice(&r.state.term.to_le_bytes());
            bytes.push(r.state.voted_for.is_some() as u8);
            bytes.extend_from_slice(&r.state.voted_for.unwrap_or(0).to_le_bytes());
            (bytes, generation)
        };
        let name = if generation % 2 == 1 { "state" } else { "snapshot" };
        let file = fs::File::create(name).await?;
        file.write_all_at(&bytes, 0).await?;
        file.sync_all().await?;
        self.inner.lock().unwrap().journal_len = generation;
        Ok(())
    }

    async fn restore(&self) -> io::Result<()> {
        let mut best: Option<(u64, u64, Option<u64>)> = None;
        let mut present = false;
        for name in ["state", "snapshot"] {
            match fs::read(name).await {
                Ok(bytes) => {
                    present = true;
                    if bytes.len() != 25 || bytes[16] > 1 { continue; }
                    let mut gen = [0;8]; gen.copy_from_slice(&bytes[..8]);
                    let mut term = [0;8]; term.copy_from_slice(&bytes[8..16]);
                    let mut vote = [0;8]; vote.copy_from_slice(&bytes[17..]);
                    let generation = u64::from_le_bytes(gen);
                    if generation == 0 { continue; }
                    if best.as_ref().map_or(true, |b| generation > b.0) {
                        best = Some((generation,u64::from_le_bytes(term),
                            if bytes[16] == 1 { Some(u64::from_le_bytes(vote)) } else { None }));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {},
                Err(e) => return Err(e),
            }
        }
        if let Some((generation,term,vote)) = best {
            let mut r = self.inner.lock().unwrap();
            r.journal_len = generation; r.state.term = term; r.state.voted_for = vote;
        } else if present {
            return Err(io::Error::new(io::ErrorKind::InvalidData,"no complete generation"));
        }
        Ok(())
    }

    fn start_timer(&self) {
        let this = self.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(25)).await;
                let gate = this.serial.lock().await;
                let (request, heartbeat) = {
                    let mut r = this.inner.lock().unwrap();
                    if matches!(r.state.role, Role::Leader) {
                        (None, Some(Heartbeat { term: r.state.term, leader: r.me }))
                    } else if Instant::now() >= r.deadline && r.state.term < u64::MAX {
                        r.state = election::timeout(r.state, r.me as u64);
                        r.voted.fill(false);
                        let me = r.me; r.voted[me] = true;
                        r.deadline = Instant::now() + Raft::generate_election_timeout();
                        let v = election::make_vote(r.state, r.me as u64, 0, 0);
                        if r.peers.len() == 1 { r.state.role = Role::Leader; }
                        (Some(RequestVoteArgs { term: v.term, candidate: v.candidate,
                            last_term: v.last_term, last_index: v.last_index }), None)
                    } else { (None, None) }
                };
                if request.is_some() { this.persist().await.expect("persist election before send"); }
                drop(gate);
                if let Some(args) = request { this.send_vote_request(args); }
                if let Some(args) = heartbeat { this.send_heartbeats(args); }
            }
        });
    }

    fn send_vote_request(&self, args: RequestVoteArgs) {
        let peers = self.inner.lock().unwrap().peers.clone();
        let me = self.inner.lock().unwrap().me;
        for (peer, addr) in peers.into_iter().enumerate() {
            if peer == me { continue; }
            let this = self.clone(); let args = args.clone();
            task::spawn(async move {
                if let Ok(reply) = this.endpoint.call_timeout(addr, args.clone(), Duration::from_millis(200)).await {
                    let _gate = this.serial.lock().await;
                    {
                        let mut r = this.inner.lock().unwrap();
                        r.state = election::advance(r.state, reply.term);
                        if r.state.term == args.term && matches!(r.state.role, Role::Candidate) && reply.granted {
                            r.voted[peer] = true;
                            if r.voted.iter().filter(|v| **v).count() > r.peers.len()/2 {
                                r.state.role = Role::Leader; r.leader = me;
                            }
                        }
                    }
                    this.persist().await.expect("persist reply term");
                }
            });
        }
    }

    fn send_heartbeats(&self, args: Heartbeat) {
        let peers = self.inner.lock().unwrap().peers.clone();
        for (peer, addr) in peers.into_iter().enumerate() {
            if peer == args.leader { continue; }
            let this = self.clone(); let args = args.clone();
            task::spawn(async move {
                if let Ok(term) = this.endpoint.call_timeout(addr, args, Duration::from_millis(150)).await {
                    let _gate = this.serial.lock().await;
                    { let mut r = this.inner.lock().unwrap(); r.state = election::advance(r.state, term); }
                    this.persist().await.expect("persist heartbeat reply");
                }
            });
        }
    }

    async fn heartbeat(&self, args: Heartbeat) -> u64 {
        let _gate = self.serial.lock().await;
        let term = {
            let mut r = self.inner.lock().unwrap();
            if args.leader >= r.peers.len() { return r.state.term; }
            r.state = election::advance(r.state, args.term);
            if args.term == r.state.term {
                r.state.role = Role::Follower; r.leader = args.leader;
                r.deadline = Instant::now() + Raft::generate_election_timeout();
            }
            r.state.term
        };
        self.persist().await.expect("persist heartbeat term before reply");
        term
    }

    fn start_rpc_server(&self) {
        let net = self.endpoint.clone();

        let this = self.clone();
        net.add_rpc_handler(move |args: RequestVoteArgs| {
            let this = this.clone();
            async move { this.request_vote(args).await.unwrap() }
        });
        let this = self.clone();
        net.add_rpc_handler(move |args: Heartbeat| {
            let this = this.clone(); async move { this.heartbeat(args).await }
        });
    }

    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        let _gate = self.serial.lock().await;
        let reply = {
            let mut this = self.inner.lock().unwrap();
            this.request_vote(args)
        };
        // if you need to persist or call async functions here,
        // make sure the lock is scoped and dropped.
        self.persist().await.expect("failed to persist");
        Ok(reply)
    }
}

// HINT: put mutable non-async functions here
impl Raft {
    fn start(&mut self, data: &[u8]) -> Result<Start> {
        if !matches!(self.state.role, Role::Leader) {
            let leader = (self.me + 1) % self.peers.len();
            return Err(Error::NotLeader(leader));
        }
        todo!("start agreement");
    }

    // Here is an example to apply committed message.
    fn apply(&self) {
        let msg = ApplyMsg::Command {
            data: todo!("apply msg"),
            index: todo!("apply msg"),
        };
        self.apply_ch.unbounded_send(msg).unwrap();
    }

    fn request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        if args.candidate >= self.peers.len() as u64 {
            return RequestVoteReply { term: self.state.term, granted: false };
        }
        let (state, reply) = election::request_vote(self.state, Vote { term: args.term,
            candidate: args.candidate, last_term: args.last_term, last_index: args.last_index }, 0, 0);
        self.state = state;
        if reply.granted { self.deadline = Instant::now() + Self::generate_election_timeout(); }
        RequestVoteReply { term: reply.term, granted: reply.granted }
    }

    // Here is an example to generate random number.
    fn generate_election_timeout() -> Duration {
        // see rand crate for more details
        Duration::from_millis(rand::thread_rng().gen_range(300..550))
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteArgs {
    term: u64, candidate: u64, last_term: u64, last_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteReply {
    term: u64, granted: bool,
}

impl net::rpc::Request for RequestVoteArgs {
    type Response = RequestVoteReply;
    const ID: u64 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Heartbeat { term: u64, leader: usize }
impl net::rpc::Request for Heartbeat { type Response = u64; const ID: u64 = 2; }
