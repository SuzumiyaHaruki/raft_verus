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
    convert::TryFrom,
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
    log: Vec<LogEntry>,
    base: usize,
    snapshot: Vec<u8>,
    commit: usize,
    applied: usize,
    next_index: Vec<usize>,
    matched: Vec<u64>,
    busy: Vec<bool>,
}

#[path = "state.rs"]
mod state;
use state::{Ballots, State};

/// Data needs to be persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Persist {
    term: u64,
    voted_for: Option<u64>,
    members: Vec<SocketAddr>,
    log: Vec<LogEntry>,
    base: usize,
    snapshot: Vec<u8>,
    commit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LogEntry { term: u64, data: Vec<u8> }

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
        if peers.is_empty() || me >= peers.len() || peers.iter().enumerate().any(|(i, peer)| peers[..i].contains(peer)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid fixed membership"));
        }
        let n = peers.len();
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
            log: vec![LogEntry { term: 0, data: Vec::new() }],
            base: 0,
            snapshot: Vec::new(),
            commit: 0,
            applied: 0,
            next_index: vec![1; n],
            matched: vec![0; n],
            busy: vec![false; n],
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
        let _serial = self.operation.lock().await;
        let result = self.inner.lock().unwrap().start(cmd)?;
        self.persist().await?;
        {
            let mut r = self.inner.lock().unwrap();
            r.advance_commit();
            r.apply();
        }
        self.send_heartbeats(result.term);
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
        raft.state.is_leader()
    }

    pub fn state_size(&self) -> usize {
        let r = self.inner.lock().unwrap();
        let image = Persist { term: r.state.term, voted_for: r.state.voted_for, members: r.peers.clone(),
            log: r.log.clone(), base: r.base, snapshot: Vec::new(), commit: r.commit };
        bincode::serialized_size(&image).unwrap_or(u64::MAX) as usize
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
        let _serial = self.operation.lock().await;
        let r = self.inner.lock().unwrap();
        r.base as u64 == last_included_index && r.log[0].term == last_included_term
            && r.snapshot == snapshot
    }

    /// The service says it has created a snapshot that has all info up to and
    /// including index. This means the service no longer needs the log through
    /// (and including) that index. Raft should now trim its log as much as
    /// possible.
    pub async fn snapshot(&self, index: u64, snapshot: &[u8]) -> Result<()> {
        let _serial = self.operation.lock().await;
        {
            let mut r = self.inner.lock().unwrap();
            let index = usize::try_from(index).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "snapshot index"))?;
            if index <= r.base { return Ok(()); }
            if !state::can_snapshot(r.base as u64, index as u64, r.applied as u64, r.commit as u64, r.last_index() as u64) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "unapplied snapshot" ).into());
            }
            let offset = index - r.base;
            r.log.drain(..offset);
            r.log[0].data.clear();
            r.base = index;
            r.snapshot = snapshot.to_vec();
        }
        self.persist().await?;
        Ok(())
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    // Caller holds operation across mutation, persistence and output release.
    // Fresh immutable records avoid truncating an acknowledged durable vote.
    async fn persist(&self) -> io::Result<()> {
        let (image, sequence) = {
            let r = self.inner.lock().unwrap();
            let image = Persist { term: r.state.term, voted_for: r.state.voted_for, members: r.peers.clone(), log: r.log.clone(), base: r.base, snapshot: r.snapshot.clone(), commit: r.commit };
            if r.saved.as_ref() == Some(&image) { return Ok(()); }
            (image, r.journal_next)
        };
        let next = sequence.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "journal exhausted"))?;
        let bytes = bincode::serialize(&image).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let path = Self::record_name(sequence);
        let file = fs::File::create(path).await?;
        file.write_all_at(&bytes, 0).await?;
        file.sync_all().await?;
        // Accounting mirrors are never used for recovery. The immutable image
        // is already complete if a crash interrupts either mirror replacement.
        let mut metadata = image.clone();
        metadata.snapshot.clear();
        let state_bytes = bincode::serialize(&metadata).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mirror = fs::File::create("state").await?;
        mirror.write_all_at(&state_bytes, 0).await?;
        mirror.sync_all().await?;
        let mirror = fs::File::create("snapshot").await?;
        mirror.write_all_at(&image.snapshot, 0).await?;
        mirror.sync_all().await?;
        if sequence > 0 {
            // In the pinned simulator set_len is supported on an opened inode.
            fs::File::open(Self::record_name(sequence-1)).await?.set_len(0).await?;
        }
        let mut r = self.inner.lock().unwrap();
        r.saved = Some(image);
        r.journal_next = next;
        Ok(())
    }

    fn record_name(sequence: u64) -> String {
        format!("image.{}", sequence)
    }

    async fn restore(&self) -> io::Result<()> {
        let mut sequence = 0u64;
        let mut previous: Option<Persist> = None;
        let mut recovered = State::fresh();
        let mut terminal_empty = false;
        loop {
            let bytes = match fs::read(Self::record_name(sequence)).await {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == io::ErrorKind::NotFound => break,
                Err(e) => return Err(e),
            };
            if bytes.is_empty() {
                terminal_empty = true;
                sequence = sequence.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "journal exhausted"))?;
                continue;
            }
            terminal_empty = false;
            // An incomplete created record halts recovery; never reinterpret it
            // as permission to reuse an unknown vote.
            let image: Persist = bincode::deserialize(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let n = self.inner.lock().unwrap().peers.len();
            if image.members != self.inner.lock().unwrap().peers {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "durable group identity mismatch"));
            }
            recovered = recovered.restore_successor(image.term, image.voted_for, n as u64)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid vote journal"))?;
            let last = image.base.checked_add(image.log.len().saturating_sub(1));
            if image.log.is_empty() || !image.log[0].data.is_empty()
                || image.base == 0 && image.log[0].term != 0
                || last.is_none() || last == Some(usize::MAX)
                || image.commit < image.base || image.commit > last.unwrap()
                || image.log.windows(2).any(|w| w[0].term > w[1].term)
                || image.log.last().unwrap().term > image.term
                || previous.as_ref().map_or(false, |old| image.commit < old.commit || image.base < old.base) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid durable log"));
            }
            if let Some(ref old) = previous {
                for i in image.base.max(old.base)..=old.commit {
                    if i > last.unwrap() || old.log[i-old.base].term != image.log[i-image.base].term
                        || i > image.base && old.log[i-old.base].data != image.log[i-image.base].data {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "changed committed prefix"));
                    }
                }
            }
            previous = Some(image);
            sequence = sequence.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "journal exhausted"))?;
        }
        if terminal_empty { return Err(io::Error::new(io::ErrorKind::InvalidData, "incomplete terminal image")); }
        for generation in 0..sequence.saturating_sub(1) {
            fs::File::open(Self::record_name(generation)).await?.set_len(0).await?;
        }
        let mut r = self.inner.lock().unwrap();
        r.state = recovered;
        if let Some(ref image) = previous {
            r.log = image.log.clone(); r.commit = image.commit; r.base = image.base;
            r.snapshot = image.snapshot.clone(); r.applied = r.base;
            if r.base > 0 {
                let _ = r.apply_ch.unbounded_send(ApplyMsg::Snapshot { data: r.snapshot.clone(), index: r.base as u64, term: r.log[0].term });
            }
        }
        r.saved = previous;
        r.journal_next = sequence;
        r.apply();
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
        net.add_rpc_handler(move |args: AppendEntries| {
            let this = this.clone();
            async move { this.append_entries(args).await.expect("append persistence") }
        });
        let this = self.clone();
        net.add_rpc_handler(move |args: InstallSnapshot| {
            let this = this.clone();
            async move { this.install_snapshot(args).await.expect("install snapshot") }
        });
    }

    async fn install_snapshot(&self, args: InstallSnapshot) -> Result<u64> {
        let _serial = self.operation.lock().await;
        let mut installed = false;
        let term = {
            let mut r = self.inner.lock().unwrap();
            if args.leader < r.peers.len() && r.state.accept_leader(args.term) {
                r.deadline = Instant::now() + Raft::generate_election_timeout();
                if let Ok(index) = usize::try_from(args.index) {
                    if index > r.commit && index < usize::MAX && args.last_term <= args.term {
                        if index <= r.last_index() && r.log[index-r.base].term == args.last_term {
                            let offset = index-r.base;
                            r.log.drain(..offset);
                            r.log[0].data.clear();
                        } else { r.log = vec![LogEntry { term: args.last_term, data: Vec::new() }]; }
                        r.base = index; r.snapshot = args.data; r.commit = index; r.applied = index;
                        installed = true;
                    }
                }
            }
            r.state.term
        };
        self.persist().await?;
        if installed {
            let r = self.inner.lock().unwrap();
            let _ = r.apply_ch.unbounded_send(ApplyMsg::Snapshot { data: r.snapshot.clone(), index: r.base as u64, term: r.log[0].term });
        }
        Ok(term)
    }

    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        let _serial = self.operation.lock().await;
        let reply = self.inner.lock().unwrap().request_vote(args);
        self.persist().await?;
        Ok(reply)
    }

    async fn append_entries(&self, args: AppendEntries) -> Result<AppendReply> {
        let _serial = self.operation.lock().await;
        let reply = self.inner.lock().unwrap().append_entries(args);
        self.persist().await?;
        self.inner.lock().unwrap().apply();
        Ok(reply)
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
        let (peers, me, last_term, last_index) = { let r = self.inner.lock().unwrap(); (r.peers.clone(), r.me, r.log.last().unwrap().term, r.last_index() as u64) };
        let this = self.clone();
        task::spawn(async move {
            let total = peers.len();
            let mut ballots = Ballots::new(term, total, me);
            let mut rpcs = FuturesUnordered::new();
            for (id, peer) in peers.into_iter().enumerate() {
                if id == me { continue; }
                let net = this.endpoint.clone();
                let args = RequestVoteArgs { term, candidate: me, last_term, last_index };
                rpcs.push(async move { (id, net.call_timeout(peer, args, Duration::from_millis(200)).await) });
            }
            {
                let _serial = this.operation.lock().await;
                this.inner.lock().unwrap().elect(&ballots);
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
                            r.elect(&ballots);
                        }
                    }
                    this.persist().await.expect("persist reply term");
                }
            }
        });
    }

    fn send_heartbeats(&self, term: u64) {
        let mut r = self.inner.lock().unwrap();
        if !r.state.is_leader() || r.state.term != term { return; }
        for id in 0..r.peers.len() {
            if id == r.me || r.busy[id] { continue; }
            if r.next_index[id] <= r.base {
                let args = InstallSnapshot { term, leader: r.me, index: r.base as u64, last_term: r.log[0].term, data: r.snapshot.clone() };
                let index = r.base;
                let peer = r.peers[id];
                r.busy[id] = true;
                let this = self.clone();
                task::spawn(async move {
                    let reply = this.endpoint.call_timeout(peer, args, Duration::from_millis(200)).await;
                    let _serial = this.operation.lock().await;
                    {
                        let mut r = this.inner.lock().unwrap(); r.busy[id] = false;
                        if let Ok(reply_term) = reply {
                            r.state.observe_term(reply_term);
                            if r.state.is_leader() && r.state.term == term && reply_term == term {
                                r.matched[id] = r.matched[id].max(index as u64);
                                r.next_index[id] = (r.matched[id] as usize).checked_add(1).expect("index exhausted");
                                r.advance_commit();
                            }
                        }
                    }
                    this.persist().await.expect("persist snapshot reply");
                    this.inner.lock().unwrap().apply();
                });
                continue;
            }
            let next = r.next_index[id].min(r.last_index()+1).max(r.base+1);
            let args = AppendEntries { term, leader: r.me, prev_index: (next - 1) as u64,
                prev_term: r.log[next - 1 - r.base].term, entries: r.log[next-r.base..].to_vec(),
                leader_commit: r.commit as u64 };
            let acknowledged = next - 1 + args.entries.len();
            let peer = r.peers[id];
            r.busy[id] = true;
            let this = self.clone();
            task::spawn(async move {
                let reply = this.endpoint.call_timeout(peer, args, Duration::from_millis(200)).await;
                let _serial = this.operation.lock().await;
                {
                    let mut r = this.inner.lock().unwrap();
                    r.busy[id] = false;
                    if let Ok(reply) = reply {
                        r.state.observe_term(reply.term);
                        if r.state.is_leader() && r.state.term == term && reply.term == term {
                            if reply.success {
                                r.matched[id] = r.matched[id].max(acknowledged as u64);
                                r.next_index[id] = usize::try_from(r.matched[id]).expect("index conversion").checked_add(1).expect("index exhausted");
                                r.advance_commit();
                            } else if r.next_index[id] == next {
                                r.next_index[id] = (reply.next as usize).min(next.saturating_sub(1)).max(1);
                            }
                        }
                    }
                }
                this.persist().await.expect("persist append reply");
                this.inner.lock().unwrap().apply();
            });
        }
    }

}

// HINT: put mutable non-async functions here
impl Raft {
    fn last_index(&self) -> usize { self.base + self.log.len() - 1 }

    fn start(&mut self, data: &[u8]) -> Result<Start> {
        if !self.state.is_leader() { return Err(Error::NotLeader((self.me + 1) % self.peers.len())); }
        let index = state::next_log_index(self.last_index().checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "index exhausted"))? as u64)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "index exhausted"))?;
        self.log.push(LogEntry { term: self.state.term, data: data.to_vec() });
        self.matched[self.me] = self.last_index() as u64;
        Ok(Start { index, term: self.state.term })
    }

    fn elect(&mut self, ballots: &Ballots) {
        if self.state.elected(ballots) {
            let next = self.last_index()+1;
            self.next_index.fill(next);
            self.matched.fill(0);
            self.matched[self.me] = self.last_index() as u64;
        }
    }

    fn advance_commit(&mut self) {
        if !self.state.is_leader() { return; }
        let mut matches = self.matched.clone();
        matches.sort_unstable();
        let candidate = matches[matches.len() - 1 - matches.len()/2];
        if candidate > self.last_index() as u64 || candidate < self.base as u64 { return; }
        self.commit = state::commit_from_matches(self.commit as u64, candidate,
            self.last_index() as u64, self.state.term, self.log[candidate as usize-self.base].term,
            &self.matched) as usize;
    }

    fn apply(&mut self) {
        while let Some(index) = state::next_application(self.applied as u64, self.commit as u64, self.last_index() as u64) {
            let msg = ApplyMsg::Command { data: self.log[index as usize-self.base].data.clone(), index };
            if self.apply_ch.unbounded_send(msg).is_err() { return; }
            self.applied = index as usize;
        }
    }

    fn append_entries(&mut self, args: AppendEntries) -> AppendReply {
        let no = |term, next| AppendReply { term, success: false, next };
        if args.leader >= self.peers.len() { return no(self.state.term, (self.last_index()+1) as u64); }
        if !self.state.accept_leader(args.term) { return no(self.state.term, (self.last_index()+1) as u64); }
        self.deadline = Instant::now() + Self::generate_election_timeout();
        let prev = match usize::try_from(args.prev_index) { Ok(v) => v, Err(_) => return no(self.state.term, (self.last_index()+1) as u64) };
        if prev < self.base { return no(self.state.term, (self.base+1) as u64); }
        if prev > self.last_index() { return no(self.state.term, (self.last_index()+1) as u64); }
        if self.log[prev-self.base].term != args.prev_term {
            let conflict = self.log[prev-self.base].term;
            let mut first = prev;
            while first > self.base && self.log[first-1-self.base].term == conflict { first -= 1; }
            return no(self.state.term, first as u64);
        }
        let through = match prev.checked_add(args.entries.len()) { Some(v) if v < usize::MAX => v, _ => return no(self.state.term, (self.last_index()+1) as u64) };
        let mut prior_term = args.prev_term;
        for entry in &args.entries {
            if entry.term < prior_term || entry.term > args.term { return no(self.state.term, (self.last_index()+1) as u64); }
            prior_term = entry.term;
        }
        for (offset, entry) in args.entries.into_iter().enumerate() {
            let index = prev + 1 + offset;
            if index <= self.last_index() && self.log[index-self.base] != entry {
                if index <= self.commit || self.log[index-self.base].term == entry.term { return no(self.state.term, index as u64); }
                self.log.truncate(index-self.base);
            }
            if index == self.last_index()+1 { self.log.push(entry); }
        }
        self.commit = state::follower_commit(self.commit as u64, args.leader_commit, through as u64) as usize;
        AppendReply { term: self.state.term, success: true, next: (through+1) as u64 }
    }

    fn request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        if args.candidate >= self.peers.len() {
            return RequestVoteReply { term: self.state.term, granted: false };
        }
        let fresh = state::log_fresh(self.log.last().unwrap().term, self.last_index() as u64, args.last_term, args.last_index);
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
    const ID: u64 = 0x5241465401;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendEntries {
    term: u64, leader: usize, prev_index: u64, prev_term: u64,
    entries: Vec<LogEntry>, leader_commit: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendReply { term: u64, success: bool, next: u64 }
impl net::rpc::Request for AppendEntries {
    type Response = AppendReply;
    const ID: u64 = 0x5241465402;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallSnapshot { term: u64, leader: usize, index: u64, last_term: u64, data: Vec<u8> }
impl net::rpc::Request for InstallSnapshot { type Response = u64; const ID: u64 = 0x5241465403; }

#[cfg(test)]
#[path = "election_checks.rs"]
mod election_checks;
