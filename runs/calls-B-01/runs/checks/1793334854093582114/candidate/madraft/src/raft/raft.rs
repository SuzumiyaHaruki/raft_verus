use futures::{channel::mpsc, stream::FuturesUnordered, StreamExt};
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
    operation: Arc<futures::lock::Mutex<()>>,
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
    state: State,
    deadline: Instant,
    journal_next: u64,
    saved: Option<Persist>,
}

#[path = "state.rs"]
mod state;
use state::{Ballots, State};

/// Data needs to be persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Persist {
    term: u64,
    voted_for: Option<u64>,
}

impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Raft({})", self.me)
    }
}

// HINT: put async functions here
impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        Self::try_new(peers, me).await.expect("initialize Raft")
    }

    async fn try_new(peers: Vec<SocketAddr>, me: usize) -> io::Result<(Self, MsgRecver)> {
        if peers.is_empty() || me >= peers.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid fixed membership"));
        }
        let endpoint = net::Endpoint::bind(peers[me]).await?;
        let (apply_ch, recver) = mpsc::unbounded();
        let inner = Arc::new(Mutex::new(Raft {
            endpoint: endpoint.clone(),
            peers,
            me,
            apply_ch,
            state: State::fresh(),
            deadline: Instant::now() + Raft::generate_election_timeout(),
            journal_next: 0,
            saved: None,
        }));
        let handle = RaftHandle { inner, endpoint, operation: Arc::new(futures::lock::Mutex::new(())) };
        // initialize from state persisted before a crash
        handle.restore().await?;
        handle.start_rpc_server();
        handle.start_ticker();

        Ok((handle, recver))
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
        raft.state.is_leader()
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
    // Caller holds operation across mutation, persistence and output release.
    // Fresh immutable records avoid truncating an acknowledged durable vote.
    async fn persist(&self) -> io::Result<()> {
        let (image, sequence) = {
            let r = self.inner.lock().unwrap();
            let image = Persist { term: r.state.term, voted_for: r.state.voted_for };
            if r.saved.as_ref() == Some(&image) { return Ok(()); }
            (image, r.journal_next)
        };
        let next = sequence.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "journal exhausted"))?;
        let bytes = bincode::serialize(&image).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let path = Self::record_name(sequence);
        let file = fs::File::create(path).await?;
        file.write_all_at(&bytes, 0).await?;
        file.sync_all().await?;
        let mut r = self.inner.lock().unwrap();
        r.saved = Some(image);
        r.journal_next = next;
        Ok(())
    }

    fn record_name(sequence: u64) -> String {
        if sequence == 0 { "state".into() } else { format!("state.{}", sequence) }
    }

    async fn restore(&self) -> io::Result<()> {
        let mut sequence = 0u64;
        let mut previous: Option<Persist> = None;
        let mut recovered = State::fresh();
        loop {
            let bytes = match fs::read(Self::record_name(sequence)).await {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == io::ErrorKind::NotFound => break,
                Err(e) => return Err(e),
            };
            // An incomplete created record halts recovery; never reinterpret it
            // as permission to reuse an unknown vote.
            let image: Persist = bincode::deserialize(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let n = self.inner.lock().unwrap().peers.len();
            recovered = recovered.restore_successor(image.term, image.voted_for, n as u64)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid vote journal"))?;
            previous = Some(image);
            sequence = sequence.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "journal exhausted"))?;
        }
        let mut r = self.inner.lock().unwrap();
        r.state = recovered;
        r.saved = previous;
        r.journal_next = sequence;
        Ok(())
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
            let this = this.clone();
            async move { this.heartbeat(args).await.expect("heartbeat persistence") }
        });
    }

    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        let _serial = self.operation.lock().await;
        let reply = self.inner.lock().unwrap().request_vote(args);
        self.persist().await?;
        Ok(reply)
    }

    async fn heartbeat(&self, args: Heartbeat) -> Result<u64> {
        let _serial = self.operation.lock().await;
        let term = {
            let mut r = self.inner.lock().unwrap();
            if args.leader < r.peers.len() && r.state.accept_leader(args.term) {
                r.deadline = Instant::now() + Raft::generate_election_timeout();
            }
            r.state.term
        };
        self.persist().await?;
        Ok(term)
    }

    fn start_ticker(&self) {
        let this = self.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(50)).await;
                let _serial = this.operation.lock().await;
                let action = {
                    let mut r = this.inner.lock().unwrap();
                    if r.state.is_leader() { Some((true, r.state.term)) }
                    else if Instant::now() >= r.deadline {
                        let me = r.me as u64;
                        r.deadline = Instant::now() + Raft::generate_election_timeout();
                        if r.state.campaign(me) { Some((false, r.state.term)) } else { None }
                    } else { None }
                };
                if let Some((leader, term)) = action {
                    this.persist().await.expect("persist before emission");
                    if leader { this.send_heartbeats(term); } else { this.send_vote_request(term); }
                }
            }
        });
    }

    fn send_vote_request(&self, term: u64) {
        let (peers, me) = { let r = self.inner.lock().unwrap(); (r.peers.clone(), r.me) };
        let this = self.clone();
        task::spawn(async move {
            let total = peers.len();
            let mut ballots = Ballots::new(term, total, me);
            let mut rpcs = FuturesUnordered::new();
            for (id, peer) in peers.into_iter().enumerate() {
                if id == me { continue; }
                let net = this.endpoint.clone();
                let args = RequestVoteArgs { term, candidate: me, last_term: 0, last_index: 0 };
                rpcs.push(async move { (id, net.call_timeout(peer, args, Duration::from_millis(200)).await) });
            }
            {
                let _serial = this.operation.lock().await;
                this.inner.lock().unwrap().state.elected(&ballots);
            }
            while let Some((peer, reply)) = rpcs.next().await {
                if let Ok(reply) = reply {
                    let _serial = this.operation.lock().await;
                    {
                        let mut r = this.inner.lock().unwrap();
                        r.state.observe_term(reply.term);
                        // One future per distinct peer; stale replies cannot
                        // contribute to a different election.
                        if r.state.term == term && reply.term == term && reply.granted {
                            ballots.record(peer, reply.term, reply.granted);
                            r.state.elected(&ballots);
                        }
                    }
                    this.persist().await.expect("persist reply term");
                }
            }
        });
    }

    fn send_heartbeats(&self, term: u64) {
        let (peers, me) = { let r = self.inner.lock().unwrap(); (r.peers.clone(), r.me) };
        for (id, peer) in peers.into_iter().enumerate() {
            if id == me { continue; }
            let this = self.clone();
            task::spawn(async move {
                if let Ok(term) = this.endpoint.call_timeout(peer, Heartbeat { term, leader: me }, Duration::from_millis(100)).await {
                    let _serial = this.operation.lock().await;
                    this.inner.lock().unwrap().state.observe_term(term);
                    this.persist().await.expect("persist heartbeat reply term");
                }
            });
        }
    }

}

// HINT: put mutable non-async functions here
impl Raft {
    fn start(&mut self, data: &[u8]) -> Result<Start> {
        if !self.state.is_leader() {
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
        if args.candidate >= self.peers.len() {
            return RequestVoteReply { term: self.state.term, granted: false };
        }
        // 2A has an empty log. 2B must replace this comparison with the
        // actual last entry and carry it in outgoing requests.
        let fresh = (args.last_term, args.last_index) >= (0, 0);
        let granted = self.state.vote(args.term, args.candidate as u64, fresh);
        if granted { self.deadline = Instant::now() + Self::generate_election_timeout(); }
        RequestVoteReply { term: self.state.term, granted }
    }

    // Here is an example to generate random number.
    fn generate_election_timeout() -> Duration {
        // see rand crate for more details
        Duration::from_millis(rand::thread_rng().gen_range(300..550))
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteArgs {
    term: u64,
    candidate: usize,
    last_term: u64,
    last_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteReply {
    term: u64,
    granted: bool,
}

impl net::rpc::Request for RequestVoteArgs {
    type Response = RequestVoteReply;
    const ID: u64 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Heartbeat { term: u64, leader: usize }
impl net::rpc::Request for Heartbeat {
    type Response = u64;
    const ID: u64 = 2;
}

#[cfg(test)]
#[path = "election_checks.rs"]
mod election_checks;
