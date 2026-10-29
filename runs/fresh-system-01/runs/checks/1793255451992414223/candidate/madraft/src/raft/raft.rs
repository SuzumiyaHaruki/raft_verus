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
#[path = "replication.rs"]
mod replication;
use replication::{Entry, Log};

struct Raft {
    peers: Vec<SocketAddr>,
    me: usize,
    apply_ch: MsgSender,
    state: Election,
    deadline: Instant,
    heartbeat_due: Instant,
    log: Log,
    next: Vec<u64>,
    matched: Vec<u64>,
    inflight: Vec<Option<u64>>,
    snapshot: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Persist {
    term: u64,
    voted_for: Option<usize>,
    entries: Vec<Entry>,
    commit: u64,
    base: u64,
    base_term: u64,
}

#[derive(Serialize, Deserialize)]
struct Image { state: Persist, snapshot: Vec<u8> }

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
            next: vec![1; peers.len()], matched: vec![0; peers.len()],
            inflight: vec![None; peers.len()], log: Log::new(), snapshot: Vec::new(),
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

    pub async fn start(&self, cmd: &[u8]) -> Result<Start> {
        let _serial = self.serial.lock().await;
        let result = {
            let mut raft = self.inner.lock().unwrap();
            if !raft.state.is_leader() { return Err(Error::NotLeader((raft.me + 1) % raft.peers.len())); }
            let term = raft.state.term;
            let index = raft.log.propose(term, cmd.to_vec()).ok_or_else(||
                io::Error::new(io::ErrorKind::Other, "log exhausted"))?;
            raft.sync_log_tip();
            raft.advance_commit();
            raft.heartbeat_due = Instant::now();
            Start { term, index }
        };
        self.persist().await?;
        self.inner.lock().unwrap().apply();
        drop(_serial);
        self.broadcast();
        Ok(result)
    }
    pub fn term(&self) -> u64 { self.inner.lock().unwrap().state.term }
    pub fn is_leader(&self) -> bool { self.inner.lock().unwrap().state.is_leader() }
    pub async fn cond_install_snapshot(&self, term: u64, index: u64, data: &[u8]) -> bool {
        let raft = self.inner.lock().unwrap();
        raft.log.base == index && raft.log.base_term == term && raft.snapshot == data
    }
    pub async fn snapshot(&self, index: u64, data: &[u8]) -> Result<()> {
        let _serial = self.serial.lock().await;
        {
            let mut raft = self.inner.lock().unwrap();
            if !raft.log.compact(index) { return Ok(()); }
            raft.snapshot = data.to_vec();
            raft.sync_log_tip();
        }
        self.persist().await?;
        Ok(())
    }
    pub async fn state_size(&self) -> usize {
        fs::metadata("state").await.map(|m| m.len() as usize).unwrap_or(0)
    }
    // The canonical full image makes state/snapshot recovery one coherent read.
    // The lab files are size-accounting mirrors; recovery never pairs them.
    async fn persist(&self) -> io::Result<()> {
        let image = {
            let raft = self.inner.lock().unwrap();
            Image { state: Persist { term: raft.state.term, voted_for: raft.state.voted_for,
                entries: raft.log.entries.clone(), commit: raft.log.commit,
                base: raft.log.base, base_term: raft.log.base_term }, snapshot: raft.snapshot.clone() }
        };
        let encode = |value: &Image| bincode::serialize(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,e));
        let full = encode(&image)?;
        let state = bincode::serialize(&image.state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,e))?;
        for (name,bytes) in [("image",full.as_slice()),("state",state.as_slice()),("snapshot",image.snapshot.as_slice())] {
            let file=fs::File::create(name).await?;
            file.write_all_at(bytes,0).await?;
            file.sync_all().await?;
        }
        Ok(())
    }
    async fn restore(&self) -> io::Result<()> {
        match fs::read("image").await {
            Ok(bytes) => {
                let image:Image=bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,e))?;
                let state=image.state;
                let log=Log::restore(state.base,state.base_term,state.entries,state.commit).ok_or_else(||
                    io::Error::new(io::ErrorKind::InvalidData,"invalid durable log"))?;
                let mut raft=self.inner.lock().unwrap();
                if !raft.state.recover(state.term,state.voted_for) {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,"invalid durable vote"));
                }
                raft.log=log;
                raft.snapshot=image.snapshot;
                raft.sync_log_tip();
                if raft.log.base>0 {
                    let _=raft.apply_ch.unbounded_send(ApplyMsg::Snapshot {data:raft.snapshot.clone(),term:raft.log.base_term,index:raft.log.base});
                }
                raft.apply();
            }
            Err(e) if e.kind()==io::ErrorKind::NotFound => {self.persist().await?;}
            Err(e)=>return Err(e),
        }
        Ok(())
    }
    fn start_rpc_server(&self) {
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: RequestVoteArgs| {
            let this = this.clone();
            async move { this.request_vote(args).await.expect("vote storage failed: node halted") }
        });
        let this=self.clone();
        self.endpoint.add_rpc_handler(move |args:SnapshotArgs| {
            let this=this.clone();
            async move {this.install_snapshot(args).await.expect("snapshot storage failed")}
        });
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: AppendArgs| {
            let this = this.clone();
            async move { this.append(args).await.expect("append storage failed: node halted") }
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
    async fn append(&self, args: AppendArgs) -> Result<AppendReply> {
        let _serial = self.serial.lock().await;
        let reply = {
            let mut raft = self.inner.lock().unwrap();
            let accepted = raft.state.heartbeat(args.term, args.leader);
            if accepted { raft.reset_deadline(); }
            let mut success = false;
            let mut hint = raft.log.last_index().saturating_add(1);
            if accepted && raft.log.term_at(args.prev_index) == Some(args.prev_term) {
                success = true;
                let mut index = args.prev_index;
                for entry in args.entries {
                    if let Some(next) = index.checked_add(1) {
                        index = next;
                        if !raft.log.append_one(index, entry) { success = false; break; }
                    } else { success = false; break; }
                }
                if success { raft.log.follower_commit(args.commit, index); }
                raft.sync_log_tip();
            } else if args.prev_index <= raft.log.last_index() && args.prev_index > 0 {
                let bad_term = raft.log.term_at(args.prev_index);
                hint = args.prev_index;
                while hint > raft.log.base.saturating_add(1) && raft.log.term_at(hint - 1) == bad_term { hint -= 1; }
            }
            AppendReply { term: raft.state.term, success, hint }
        };
        self.persist().await?;
        self.inner.lock().unwrap().apply();
        Ok(reply)
    }
    async fn install_snapshot(&self,args:SnapshotArgs)->Result<AppendReply> {
        let _serial=self.serial.lock().await;
        let (term,installed)={
            let mut raft=self.inner.lock().unwrap();
            let accepted=raft.state.heartbeat(args.term,args.leader);
            if accepted {raft.reset_deadline();}
            let installed=accepted && raft.log.install(args.index,args.last_term);
            if installed {raft.snapshot=args.data.clone();raft.sync_log_tip();}
            (raft.state.term,installed)
        };
        self.persist().await?;
        if installed {
            let raft=self.inner.lock().unwrap();
            let _=raft.apply_ch.unbounded_send(ApplyMsg::Snapshot {data:args.data,term:args.last_term,index:args.index});
        }
        Ok(AppendReply{term,success:true,hint:0})
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
        let votes = {
            let mut raft = self.inner.lock().unwrap();
            let now = Instant::now();
            if raft.state.is_leader() {
                if now < raft.heartbeat_due { return; }
                raft.heartbeat_due = now + Duration::from_millis(80);
                None
            } else {
                if now < raft.deadline { return; }
                raft.reset_deadline();
                if !raft.state.campaign() { return; }
                let args = RequestVoteArgs { term: raft.state.term, candidate: raft.me,
                    last_term: raft.state.last_term, last_index: raft.state.last_index };
                let me = raft.me;
                if raft.state.receive_grant(args.term, me, true) { raft.became_leader(); }
                Some(args)
            }
        };
        self.persist().await.expect("timer storage failed: node halted");
        drop(_serial);
        if let Some(args) = votes {
            let peers = self.inner.lock().unwrap().peers.clone();
            for (peer, addr) in peers.into_iter().enumerate() {
                if peer == args.candidate { continue; }
                let this = self.clone();
                let args = args.clone();
                task::spawn(async move {
                    if let Ok(reply) = this.endpoint.call_timeout(addr, args, Duration::from_millis(250)).await {
                        this.vote_reply(peer, reply).await;
                    }
                });
            }
        } else { self.broadcast(); }
    }
    fn broadcast(&self) {
        let requests = {
            let mut raft = self.inner.lock().unwrap();
            if !raft.state.is_leader() { return; }
            let mut requests = Vec::new();
            for peer in 0..raft.peers.len() {
                if peer == raft.me || raft.inflight[peer].is_some() { continue; }
                let next = raft.next[peer];
                if next == 0 { continue; }
                if next <= raft.log.base {
                    let args=SnapshotArgs {term:raft.state.term,leader:raft.me,index:raft.log.base,
                        last_term:raft.log.base_term,data:raft.snapshot.clone()};
                    raft.inflight[peer]=Some(args.term);
                    let this=self.clone();let addr=raft.peers[peer];
                    task::spawn(async move {
                        let reply=this.endpoint.call_timeout(addr,args.clone(),Duration::from_millis(250)).await.ok();
                        this.snapshot_reply(peer,args,reply).await;
                    });
                    continue;
                }
                let prev_index = next - 1;
                if let Some(prev_term) = raft.log.term_at(prev_index) {
                    let args = AppendArgs { term: raft.state.term, leader: raft.me, prev_index, prev_term,
                        entries: raft.log.entries[(prev_index-raft.log.base) as usize..].to_vec(), commit: raft.log.commit };
                    raft.inflight[peer] = Some(args.term);
                    requests.push((peer, raft.peers[peer], args));
                }
            }
            requests
        };
        for (peer, addr, args) in requests {
            let this = self.clone();
            task::spawn(async move {
                let reply = this.endpoint.call_timeout(addr, args.clone(), Duration::from_millis(250)).await.ok();
                this.append_reply(peer, args, reply).await;
            });
        }
    }
    async fn snapshot_reply(&self,peer:usize,args:SnapshotArgs,reply:Option<AppendReply>) {
        let _serial=self.serial.lock().await;
        {
            let mut raft=self.inner.lock().unwrap();
            if raft.inflight[peer]==Some(args.term) {raft.inflight[peer]=None;}
            if let Some(reply)=reply {
                raft.state.observe_term(reply.term);
                if raft.state.is_leader() && raft.state.term==args.term && reply.term==args.term {
                    raft.matched[peer]=raft.matched[peer].max(args.index);
                    raft.next[peer]=raft.matched[peer].saturating_add(1);
                    raft.heartbeat_due=Instant::now();
                }
            }
        }
        self.persist().await.expect("snapshot reply storage failed");
    }
    async fn append_reply(&self, peer: usize, args: AppendArgs, reply: Option<AppendReply>) {
        let _serial = self.serial.lock().await;
        {
            let mut raft = self.inner.lock().unwrap();
            if peer >= raft.peers.len() { return; }
            if raft.inflight[peer] == Some(args.term) { raft.inflight[peer] = None; }
            if let Some(reply) = reply {
                raft.state.observe_term(reply.term);
                if raft.state.is_leader() && raft.state.term == args.term && reply.term == args.term {
                    if reply.success {
                        if let Some(last) = args.prev_index.checked_add(args.entries.len() as u64) {
                            raft.matched[peer] = raft.matched[peer].max(last);
                            raft.next[peer] = last.saturating_add(1);
                            raft.advance_commit();
                        }
                    } else {
                        raft.next[peer] = reply.hint.max(1).min(raft.next[peer].saturating_sub(1).max(1));
                        raft.heartbeat_due = Instant::now();
                    }
                }
            }
        }
        self.persist().await.expect("append reply storage failed: node halted");
        self.inner.lock().unwrap().apply();
    }
    async fn vote_reply(&self, peer: usize, reply: RequestVoteReply) {
        let _serial = self.serial.lock().await;
        {
            let mut raft = self.inner.lock().unwrap();
            if raft.state.receive_grant(reply.term, peer, reply.granted) {
                raft.became_leader();
            }
        }
        self.persist().await.expect("reply storage failed: node halted");
    }

}
impl Raft {
    fn sync_log_tip(&mut self) {
        self.log.sync_election(&mut self.state);
    }
    fn became_leader(&mut self) {
        let next = self.log.last_index().saturating_add(1);
        self.next.fill(next);
        self.matched.fill(0);
        self.inflight.fill(None);
        self.heartbeat_due = Instant::now();
        self.advance_commit();
    }
    fn advance_commit(&mut self) { self.log.leader_commit(self.state.term, self.me, &self.matched); }
    fn apply(&mut self) {
        while let Some((index, entry)) = self.log.next_application() {
            if self.apply_ch.unbounded_send(ApplyMsg::Command { data: entry.data, index }).is_err() { break; }
        }
    }
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
struct AppendArgs { term: u64, leader: usize, prev_index: u64, prev_term: u64, entries: Vec<Entry>, commit: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppendReply { term: u64, success: bool, hint: u64 }
impl net::rpc::Request for AppendArgs {
    type Response = AppendReply;
    const ID: u64 = 10;
}

#[derive(Debug,Clone,Serialize,Deserialize)]
struct SnapshotArgs {term:u64,leader:usize,index:u64,last_term:u64,data:Vec<u8>}
impl net::rpc::Request for SnapshotArgs {type Response=AppendReply;const ID:u64=11;}
