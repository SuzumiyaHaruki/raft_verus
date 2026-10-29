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
    log: Vec<Entry>,
    commit: u64,
    applied: u64,
    next: Vec<usize>,
    matched: Vec<usize>,
    busy: Vec<bool>,
    heartbeat_at: Instant,
}

#[path = "election.rs"]
mod election;
use election::{Election, Role, Vote};
#[path = "log.rs"]
mod log_core;
use log_core::Entry;

#[derive(Serialize, Deserialize)]
struct Durable { generation: u64, term: u64, vote: Option<u64>, log: Vec<Entry> }


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
            log: Vec::new(), commit: 0, applied: 0,
            next: vec![1;peers.len()], matched: vec![0;peers.len()], busy: vec![false;peers.len()],
            heartbeat_at: Instant::now(),
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
        let _gate = self.serial.lock().await;
        let result = { self.inner.lock().unwrap().start(cmd)? };
        self.persist().await?;
        { let mut r = self.inner.lock().unwrap(); r.advance_commit(); r.apply(); }
        drop(_gate);
        self.replicate();
        Ok(result)
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
            let bytes = bincode::serialize(&Durable { generation, term: r.state.term,
                vote: r.state.voted_for, log: r.log.clone() })
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,e))?;
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
        let mut best: Option<Durable> = None;
        let mut present = false;
        for name in ["state", "snapshot"] {
            match fs::read(name).await {
                Ok(bytes) => {
                    present = true;
                    if let Ok(image) = bincode::deserialize::<Durable>(&bytes) {
                        if image.generation > 0 && best.as_ref().map_or(true, |b| image.generation > b.generation) {
                            best = Some(image);
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {},
                Err(e) => return Err(e),
            }
        }
        if let Some(image) = best {
            let mut r = self.inner.lock().unwrap();
            r.journal_len = image.generation; r.state.term = image.term;
            r.state.voted_for = image.vote; r.log = image.log;
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
                    if matches!(r.state.role, Role::Leader) && Instant::now() >= r.heartbeat_at {
                        r.heartbeat_at = Instant::now() + Duration::from_millis(100);
                        (None, true)
                    } else if !matches!(r.state.role,Role::Leader) && Instant::now() >= r.deadline && r.state.term < u64::MAX {
                        r.state = election::timeout(r.state, r.me as u64);
                        r.voted.fill(false);
                        let me = r.me; r.voted[me] = true;
                        r.deadline = Instant::now() + Raft::generate_election_timeout();
                        let v = election::make_vote(r.state, r.me as u64, r.last_term(), r.log.len() as u64);
                        if r.peers.len() == 1 { r.become_leader(); }
                        (Some(RequestVoteArgs { term: v.term, candidate: v.candidate,
                            last_term: v.last_term, last_index: v.last_index }), false)
                    } else { (None, false) }
                };
                if request.is_some() { this.persist().await.expect("persist election before send"); }
                drop(gate);
                if let Some(args) = request { this.send_vote_request(args); }
                if heartbeat { this.replicate(); }
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
                            let votes = r.voted.iter().filter(|v| **v).count();
                            r.state = election::reply(r.state,reply.term,args.term,reply.granted,votes,r.peers.len());
                            if matches!(r.state.role,Role::Leader) { r.become_leader(); }
                        }
                    }
                    this.persist().await.expect("persist reply term");
                }
            });
        }
    }

    fn replicate(&self) {
        let requests = {
            let mut r = self.inner.lock().unwrap();
            if !matches!(r.state.role,Role::Leader) { return; }
            let mut requests = Vec::new();
            for peer in 0..r.peers.len() {
                if peer == r.me || r.busy[peer] { continue; }
                let next = r.next[peer].max(1).min(r.log.len()+1);
                let prev = next-1;
                let args = AppendArgs { term:r.state.term, leader:r.me, prev:prev as u64,
                    prev_term: if prev==0 {0} else {r.log[prev-1].term},
                    entries:r.log[prev..].to_vec(), commit:r.commit };
                r.busy[peer] = true;
                requests.push((peer,r.peers[peer],args));
            }
            requests
        };
        for (peer,addr,args) in requests {
            let this = self.clone();
            task::spawn(async move {
                let result = this.endpoint.call_timeout(addr,args.clone(),Duration::from_millis(200)).await;
                let _gate = this.serial.lock().await;
                {
                    let mut r = this.inner.lock().unwrap();
                    r.busy[peer] = false;
                    if let Ok(reply) = result {
                        r.state = election::advance(r.state,reply.term);
                        if r.state.term == args.term && matches!(r.state.role,Role::Leader) {
                            if reply.ok {
                                let matched = args.prev as usize + args.entries.len();
                                r.matched[peer] = r.matched[peer].max(matched);
                                r.next[peer] = r.next[peer].max(matched+1);
                                r.advance_commit();
                            } else if r.next[peer] == args.prev as usize + 1 {
                                r.next[peer] = (reply.next as usize).max(1).min(r.log.len()+1);
                            }
                        }
                    }
                }
                this.persist().await.expect("persist replication reply");
                this.inner.lock().unwrap().apply();
            });
        }
    }

    async fn append_entries(&self, args: AppendArgs) -> AppendReply {
        let _gate = self.serial.lock().await;
        let reply = {
            let mut r = self.inner.lock().unwrap();
            r.append_entries(args)
        };
        self.persist().await.expect("persist append before reply or application");
        self.inner.lock().unwrap().apply();
        reply
    }

    fn start_rpc_server(&self) {
        let net = self.endpoint.clone();

        let this = self.clone();
        net.add_rpc_handler(move |args: RequestVoteArgs| {
            let this = this.clone();
            async move { this.request_vote(args).await.unwrap() }
        });
        let this = self.clone();
        net.add_rpc_handler(move |args: AppendArgs| {
            let this = this.clone(); async move { this.append_entries(args).await }
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
        let term = self.state.term;
        if !log_core::propose(&mut self.log, Entry {term, command:data.to_vec()}) {
            return Err(io::Error::new(io::ErrorKind::Other,"log exhausted").into());
        }
        self.matched[self.me] = self.log.len();
        Ok(Start {index:self.log.len() as u64,term})
    }

    fn last_term(&self) -> u64 { self.log.last().map_or(0,|e|e.term) }
    fn become_leader(&mut self) {
        self.state.role = Role::Leader; self.leader = self.me;
        self.next.fill(self.log.len()+1); self.matched.fill(0); self.busy.fill(false);
        self.matched[self.me] = self.log.len(); self.heartbeat_at = Instant::now();
    }
    fn advance_commit(&mut self) {
        if !matches!(self.state.role,Role::Leader) { return; }
        for i in ((self.commit as usize+1)..=self.log.len()).rev() {
            let count = self.matched.iter().filter(|m| **m >= i).count();
            let next = log_core::commit_candidate(self.commit,i as u64,self.log[i-1].term,self.state.term,count,self.peers.len());
            if next > self.commit { self.commit = next; break; }
        }
    }
    fn apply(&mut self) {
        while let Some(index) = log_core::next_application(self.applied,self.commit,self.log.len()) {
            let data = self.log[index as usize-1].command.clone();
            if self.apply_ch.unbounded_send(ApplyMsg::Command {data,index}).is_err() { return; }
            self.applied = index;
        }
    }
    fn append_entries(&mut self,args:AppendArgs) -> AppendReply {
        let reject = |term,next| AppendReply {term,ok:false,next};
        if args.leader >= self.peers.len() || args.term < self.state.term {
            return reject(self.state.term,self.log.len() as u64+1);
        }
        self.state = election::heartbeat(self.state,args.term);
        self.leader = args.leader;
        self.deadline = Instant::now()+Self::generate_election_timeout();
        if args.prev > self.log.len() as u64 { return reject(self.state.term,self.log.len() as u64+1); }
        let prev = args.prev as usize;
        if !log_core::matches(&self.log,prev,args.prev_term) {
            let conflict_term = self.log[prev-1].term;
            let mut first = prev-1;
            while first > 0 && self.log[first-1].term == conflict_term { first -= 1; }
            return reject(self.state.term,first as u64+1);
        }
        if args.entries.len() > usize::MAX-prev { return reject(self.state.term,self.log.len() as u64+1); }
        let count = args.entries.len();
        for (offset,entry) in args.entries.into_iter().enumerate() {
            if !log_core::insert(&mut self.log,prev+offset,entry) { return reject(self.state.term,self.log.len() as u64+1); }
        }
        self.commit = self.commit.max(args.commit.min((prev+count) as u64));
        AppendReply {term:self.state.term,ok:true,next:(prev+count) as u64+1}
    }

    fn request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        if args.candidate >= self.peers.len() as u64 {
            return RequestVoteReply { term: self.state.term, granted: false };
        }
        let (state, reply) = election::request_vote(self.state, Vote { term: args.term,
            candidate: args.candidate, last_term: args.last_term, last_index: args.last_index }, self.last_term(), self.log.len() as u64);
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
struct AppendArgs { term:u64,leader:usize,prev:u64,prev_term:u64,entries:Vec<Entry>,commit:u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendReply { term:u64,ok:bool,next:u64 }
impl net::rpc::Request for AppendArgs { type Response = AppendReply; const ID:u64 = 2; }
