use futures::{channel::mpsc, lock::Mutex as AsyncMutex};
use madsim::{fs, net, rand::{self, Rng}, task, time::*};
use serde::{Deserialize, Serialize};
use std::{fmt, io, net::SocketAddr, sync::{Arc, Mutex}};

#[path = "state.rs"]
mod state;
use state::{State, Role};

#[derive(Clone)]
pub struct RaftHandle {
    pub(crate) endpoint: net::Endpoint,
    inner: Arc<Mutex<Raft>>,
    gate: Arc<AsyncMutex<()>>,
}
type MsgSender = mpsc::UnboundedSender<ApplyMsg>;
pub type MsgRecver = mpsc::UnboundedReceiver<ApplyMsg>;
pub enum ApplyMsg {
    Command { data: Vec<u8>, index: u64 },
    Snapshot { data: Vec<u8>, term: u64, index: u64 },
}
#[derive(Debug)]
pub struct Start { pub index: u64, pub term: u64 }
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("this node is not a leader, next leader: {0}")]
    NotLeader(usize),
    #[error("IO error")]
    IO(#[from] io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry { term: u64, data: Vec<u8> }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Persist {
    generation: u64, term: u64, vote: Option<u64>,
    base: u64, base_term: u64, log: Vec<Entry>,
    #[serde(skip)]
    snapshot: Vec<u8>,
}
struct Raft {
    peers: Vec<SocketAddr>, me: usize, apply_ch: MsgSender, state: State,
    disk: Persist, slots: [u64; 2], commit: u64, applied: u64,
    deadline: Instant, heartbeat: Instant, votes: Vec<bool>, next: Vec<u64>, matched: Vec<u64>,
}
fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, message) }
impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Raft({})", self.me) }
}
impl Raft {
    fn timeout() -> Duration { Duration::from_millis(rand::thread_rng().gen_range(350..650)) }
    fn reset(&mut self) { self.deadline = Instant::now() + Self::timeout(); }
    fn last(&self) -> u64 { self.disk.base + self.disk.log.len() as u64 }
    fn term_at(&self, index: u64) -> Option<u64> {
        if index == self.disk.base { Some(self.disk.base_term) }
        else if index > self.disk.base { self.disk.log.get((index - self.disk.base - 1) as usize).map(|e| e.term) }
        else { None }
    }
    fn apply(&mut self) {
        while self.applied < self.commit {
            self.applied += 1;
            let entry = &self.disk.log[(self.applied - self.disk.base - 1) as usize];
            let _ = self.apply_ch.unbounded_send(ApplyMsg::Command { data: entry.data.clone(), index: self.applied });
        }
    }
    fn advance_commit(&mut self) {
        let mut m = self.matched.clone();
        m[self.me] = self.last();
        m.sort_unstable();
        let index = m[(m.len() - 1) / 2];
        if index > self.commit && self.term_at(index) == Some(self.state.term) { self.commit = index; }
        self.apply();
    }
    fn become_leader(&mut self) {
        self.state.role = Role::Leader;
        let next = self.last().saturating_add(1);
        self.next.fill(next);
        self.matched.fill(0);
        self.heartbeat = Instant::now();
    }
}
impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        assert!(!peers.is_empty() && me < peers.len(), "invalid fixed membership");
        let endpoint = net::Endpoint::bind(peers[me]).await.expect("bind Raft RPC");
        let (apply_ch, recver) = mpsc::unbounded();
        let n = peers.len();
        let this = Self { endpoint, gate: Arc::new(AsyncMutex::new(())), inner: Arc::new(Mutex::new(Raft {
            peers, me, apply_ch, state: State::default(),
            disk: Persist { generation: 0, term: 0, vote: None, base: 0, base_term: 0, log: vec![], snapshot: vec![] },
            slots: [0, 0], commit: 0, applied: 0,
            deadline: Instant::now() + Raft::timeout(), heartbeat: Instant::now(),
            votes: vec![false; n], next: vec![1; n], matched: vec![0; n],
        })) };
        this.restore().await.expect("failed to restore");
        this.start_rpc_server();
        let h = this.clone();
        task::spawn(async move { loop { sleep(Duration::from_millis(10)).await; h.tick().await; } });
        (this, recver)
    }
    pub fn term(&self) -> u64 { self.inner.lock().unwrap().state.term }
    pub fn is_leader(&self) -> bool { self.inner.lock().unwrap().state.is_leader() }
    pub async fn start(&self, cmd: &[u8]) -> Result<Start> {
        let _gate = self.gate.lock().await;
        let result = {
            let mut r = self.inner.lock().unwrap();
            if !r.state.is_leader() { return Err(Error::NotLeader((r.me + 1) % r.peers.len())); }
            if r.last() == u64::MAX { return Err(invalid("log exhausted").into()); }
            let term = r.state.term;
            r.disk.log.push(Entry { term, data: cmd.to_vec() });
            r.heartbeat = Instant::now();
            Start { index: r.last(), term }
        };
        self.persist().await?;
        self.inner.lock().unwrap().advance_commit();
        Ok(result)
    }
    pub async fn snapshot(&self, index: u64, snapshot: &[u8]) -> Result<()> {
        let _gate = self.gate.lock().await;
        {
            let mut r = self.inner.lock().unwrap();
            if index <= r.disk.base { return Ok(()); }
            if index > r.applied { return Err(invalid("snapshot beyond applied prefix").into()); }
            let term = r.term_at(index).ok_or_else(|| invalid("snapshot boundary"))?;
            let remove = (index - r.disk.base) as usize;
            r.disk.log.drain(..remove);
            r.disk.base = index; r.disk.base_term = term; r.disk.snapshot = snapshot.to_vec();
        }
        self.persist().await?;
        Ok(())
    }
    pub async fn cond_install_snapshot(&self, term: u64, index: u64, snapshot: &[u8]) -> bool {
        let _gate = self.gate.lock().await;
        let r = self.inner.lock().unwrap();
        r.disk.base == index && r.disk.base_term == term && r.disk.snapshot == snapshot
    }
    // Two generation-matched state/snapshot pairs; replace the older pair first.
    // Recovery accepts only complete pairs. No cross-file atomicity is required.
    async fn persist(&self) -> io::Result<()> {
        let (bytes, snapshot, first, generation) = {
            let mut r = self.inner.lock().unwrap();
            r.disk.term = r.state.term; r.disk.vote = r.state.vote;
            r.disk.generation = r.disk.generation.checked_add(1).ok_or_else(|| invalid("generation exhausted"))?;
            (bincode::serialize(&r.disk).map_err(|_| invalid("encode state"))?,
                bincode::serialize(&(r.disk.generation, &r.disk.snapshot)).map_err(|_| invalid("encode snapshot"))?,
                if r.slots[0] <= r.slots[1] { 0 } else { 1 }, r.disk.generation)
        };
        for slot in [first, 1 - first] {
            // Invalidate/replace the older pair first; the other pair remains recoverable.
            let snap = fs::File::create(if slot == 0 { "snapshot" } else { "snapshot.backup" }).await?;
            snap.write_all_at(&snapshot, 0).await?;
            snap.sync_all().await?;
            let file = fs::File::create(if slot == 0 { "state" } else { "state.backup" }).await?;
            file.write_all_at(&bytes, 0).await?;
            file.sync_all().await?;
            self.inner.lock().unwrap().slots[slot] = generation;
        }
        Ok(())
    }
    async fn restore(&self) -> io::Result<()> {
        let mut best: Option<Persist> = None;
        let mut exists = false;
        for (slot, name) in ["state", "state.backup"].iter().enumerate() {
            match fs::read(name).await {
                Ok(bytes) => {
                    exists = true;
                    if let Ok(mut p) = bincode::deserialize::<Persist>(&bytes) {
                        let snapshot = fs::read(if slot == 0 { "snapshot" } else { "snapshot.backup" }).await;
                        let decoded = snapshot.ok().and_then(|b| bincode::deserialize::<(u64, Vec<u8>)>(&b).ok());
                        match decoded {
                            Some((generation, data)) if generation == p.generation => p.snapshot = data,
                            _ => continue,
                        }
                        self.inner.lock().unwrap().slots[slot] = p.generation;
                        if best.as_ref().map_or(true, |b| p.generation > b.generation) { best = Some(p); }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {},
                Err(e) => return Err(e),
            }
        }
        if exists && best.is_none() { return Err(invalid("no complete durable image; refusing fresh vote")); }
        if let Some(p) = best {
            let mut r = self.inner.lock().unwrap();
            r.state.term = p.term; r.state.vote = p.vote;
            r.commit = p.base; r.applied = p.base;
            if p.base > 0 { let _ = r.apply_ch.unbounded_send(ApplyMsg::Snapshot { data: p.snapshot.clone(), term: p.base_term, index: p.base }); }
            r.disk = p;
        }
        self.persist().await
    }
    fn start_rpc_server(&self) {
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |a: RequestVoteArgs| { let h = this.clone(); async move { h.request_vote(a).await.expect("vote persistence") } });
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |a: AppendArgs| { let h = this.clone(); async move { h.append(a).await.expect("append persistence") } });
    }
    async fn request_vote(&self, a: RequestVoteArgs) -> io::Result<RequestVoteReply> {
        let _gate = self.gate.lock().await;
        let reply = {
            let mut r = self.inner.lock().unwrap();
            let fresh = (a.last_term, a.last) >= (r.term_at(r.last()).unwrap(), r.last());
            let valid = a.candidate < r.peers.len() as u64;
            let granted = if valid { r.state.grant_vote(a.term, a.candidate, fresh) } else { false };
            if granted { r.reset(); }
            RequestVoteReply { term: r.state.term, granted }
        };
        self.persist().await?;
        Ok(reply)
    }
    async fn append(&self, a: AppendArgs) -> io::Result<AppendReply> {
        let mut installed = false;
        let _gate = self.gate.lock().await;
        let reply = {
            let mut r = self.inner.lock().unwrap();
            let mut success = false;
            let mut hint = r.last().saturating_add(1);
            if a.leader < r.peers.len() && a.term >= r.state.term {
                r.state.adopt_term(a.term); r.state.role = Role::Follower; r.reset();
                if let Some((base, term, bytes)) = a.snapshot {
                    if base > r.commit {
                        if r.term_at(base) == Some(term) {
                            let remove = (base - r.disk.base) as usize;
                            r.disk.log.drain(..remove);
                        } else { r.disk.log.clear(); }
                        r.disk.base = base; r.disk.base_term = term; r.disk.snapshot = bytes.clone();
                        r.commit = base; r.applied = base; installed = true;
                        // Released after the durable write below.
                    }
                    success = true;
                } else if r.term_at(a.prev) == Some(a.prev_term) {
                    success = true;
                    let mut index = a.prev;
                    for entry in a.entries {
                        if index == u64::MAX { success = false; break; }
                        index += 1;
                        if r.term_at(index) != Some(entry.term) {
                            if index <= r.commit { success = false; break; }
                            let keep = (index - r.disk.base - 1) as usize;
                            r.disk.log.truncate(keep); r.disk.log.push(entry);
                        }
                    }
                    if success { r.commit = r.commit.max(a.commit.min(index)); }
                } else if let Some(term) = r.term_at(a.prev) {
                    hint = a.prev;
                    while hint > r.disk.base && r.term_at(hint - 1) == Some(term) { hint -= 1; }
                }
            }
            AppendReply { term: r.state.term, success, hint }
        };
        self.persist().await?;
        let mut r = self.inner.lock().unwrap();
        if installed {
            let _ = r.apply_ch.unbounded_send(ApplyMsg::Snapshot { data: r.disk.snapshot.clone(), term: r.disk.base_term, index: r.disk.base });
        }
        r.apply();
        Ok(reply)
    }
    async fn tick(&self) {
        let _gate = self.gate.lock().await;
        let election = {
            let mut r = self.inner.lock().unwrap();
            if !r.state.is_leader() && Instant::now() >= r.deadline {
                let me = r.me;
                if !r.state.campaign(me as u64) { r.reset(); return; }
                r.votes.fill(false); r.votes[me] = true; r.reset();
                let a = RequestVoteArgs { term: r.state.term, candidate: me as u64, last: r.last(), last_term: r.term_at(r.last()).unwrap() };
                if r.peers.len() == 1 { r.become_leader(); }
                Some((a, r.peers.clone(), me))
            } else { None }
        };
        if let Some((a, peers, me)) = election {
            self.persist().await.expect("campaign persistence");
            for (peer, addr) in peers.into_iter().enumerate().filter(|(i,_)| *i != me) {
                let h = self.clone(); let a = a.clone();
                task::spawn(async move {
                    if let Ok(reply) = h.endpoint.call_timeout(addr, a.clone(), Duration::from_millis(250)).await {
                        h.vote_reply(peer, a.term, reply).await;
                    }
                });
            }
        }
        let sends = {
            let mut r = self.inner.lock().unwrap();
            let mut sends = vec![];
            if r.state.is_leader() && Instant::now() >= r.heartbeat {
                r.heartbeat = Instant::now() + Duration::from_millis(80);
                for peer in 0..r.peers.len() {
                    if peer == r.me { continue; }
                    let next = r.next[peer].min(r.last().saturating_add(1));
                    let snapshot = if next <= r.disk.base { Some((r.disk.base, r.disk.base_term, r.disk.snapshot.clone())) } else { None };
                    let prev = if snapshot.is_some() { r.disk.base } else { next - 1 };
                    let entries = if snapshot.is_some() { vec![] } else { r.disk.log[(next - r.disk.base - 1) as usize..].to_vec() };
                    sends.push((peer, r.peers[peer], AppendArgs { term: r.state.term, leader: r.me, prev, prev_term: r.term_at(prev).unwrap(), entries, commit: r.commit, snapshot }));
                }
            }
            sends
        };
        for (peer, addr, a) in sends {
            let h = self.clone();
            task::spawn(async move {
                if let Ok(reply) = h.endpoint.call_timeout(addr, a.clone(), Duration::from_millis(250)).await { h.append_reply(peer, a, reply).await; }
            });
        }
    }
    async fn vote_reply(&self, peer: usize, term: u64, reply: RequestVoteReply) {
        let _gate = self.gate.lock().await;
        {
            let mut r = self.inner.lock().unwrap();
            r.state.adopt_term(reply.term);
            if r.state.term == term && r.state.role == Role::Candidate && reply.granted {
                r.votes[peer] = true;
                if r.votes.iter().filter(|v| **v).count() > r.peers.len() / 2 { r.become_leader(); }
            }
        }
        self.persist().await.expect("reply persistence");
    }
    async fn append_reply(&self, peer: usize, a: AppendArgs, reply: AppendReply) {
        let _gate = self.gate.lock().await;
        {
            let mut r = self.inner.lock().unwrap();
            r.state.adopt_term(reply.term);
            if r.state.is_leader() && r.state.term == a.term {
                if reply.success {
                    let matched = a.snapshot.as_ref().map_or(a.prev + a.entries.len() as u64, |s| s.0);
                    r.matched[peer] = r.matched[peer].max(matched);
                    r.next[peer] = r.matched[peer].saturating_add(1);
                    r.advance_commit();
                } else if r.next[peer] == a.prev.saturating_add(1) {
                    r.next[peer] = reply.hint.min(r.next[peer].saturating_sub(1)).max(1);
                    r.heartbeat = Instant::now();
                }
            }
        }
        self.persist().await.expect("reply persistence");
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteArgs { term: u64, candidate: u64, last: u64, last_term: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestVoteReply { term: u64, granted: bool }
impl net::rpc::Request for RequestVoteArgs { type Response = RequestVoteReply; const ID: u64 = 1; }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendArgs { term: u64, leader: usize, prev: u64, prev_term: u64, entries: Vec<Entry>, commit: u64, snapshot: Option<(u64,u64,Vec<u8>)> }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendReply { term: u64, success: bool, hint: u64 }
impl net::rpc::Request for AppendArgs { type Response = AppendReply; const ID: u64 = 10; }
