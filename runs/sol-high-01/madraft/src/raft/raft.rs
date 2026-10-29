use futures::{channel::mpsc, lock::Mutex as AsyncMutex};
use madsim::{fs, net, rand::{self, Rng}, task, time::*};
use serde::{Deserialize, Serialize};
use std::{fmt, io, net::SocketAddr, sync::{Arc, Mutex}};

#[path = "state.rs"]
mod state;
#[path = "core.rs"]
mod core;
#[path = "log.rs"]
mod log;
#[path = "quorum.rs"]
mod quorum;
#[path = "cursor.rs"]
mod cursor;
#[path = "storage.rs"]
mod storage;
use core::*;
use state::State;

#[derive(Clone)]
pub struct RaftHandle {
    pub(crate) endpoint: net::Endpoint,
    inner: Arc<Mutex<Raft>>,
    // A transition, its persistence, and publication form one serialized host
    // operation. A crash may interrupt any await; recovery uses complete images.
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

struct Raft {
    peers: Vec<SocketAddr>,
    me: usize,
    apply_ch: MsgSender,
    core: Core,
    published: State,
    deadline: Instant,
    heartbeat: Instant,
    inflight: Vec<bool>,
    generation: u64,
    failed: bool,
}
impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Raft({})", self.me) }
}

#[derive(Clone, Serialize, Deserialize)]
struct Persist {
    generation: u64,
    peers: Vec<SocketAddr>,
    me: usize,
    durable: Durable,
}
#[derive(Serialize, Deserialize)]
struct Image { state: Persist, snapshot: Vec<u8> }
#[derive(Serialize, Deserialize)]
struct SnapshotImage { generation: u64, data: Vec<u8> }

fn invalid(reason: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, reason) }
async fn read_optional(path: &str) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path).await {
        Ok(data) => Ok(Some(data)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
async fn write_file(path: &str, bytes: &[u8]) -> io::Result<()> {
    let file = fs::File::create(path).await?;
    file.write_all_at(bytes, 0).await?;
    file.sync_all().await
}

fn encode_projections(image: &Image) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let state = bincode::serialize(&image.state).map_err(|_| invalid("encode state"))?;
    let snapshot = if image.state.durable.base == 0 { vec![] } else {
        bincode::serialize(&SnapshotImage { generation: image.state.generation, data: image.snapshot.clone() })
            .map_err(|_| invalid("encode snapshot"))?
    };
    Ok((state, snapshot))
}

impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        // Invalid deployment cannot emit protocol messages.
        assert!(!peers.is_empty() && me < peers.len(), "invalid Raft membership");
        assert!(!peers.iter().enumerate().any(|(i, p)| peers[..i].contains(p)), "duplicate Raft member");
        let endpoint = net::Endpoint::bind(peers[me]).await.expect("bind Raft RPC");
        let (apply_ch, recver) = mpsc::unbounded();
        let members = peers.len();
        let inner = Arc::new(Mutex::new(Raft {
            peers, me, apply_ch, core: Core::new(me, members), published: State::initial(),
            deadline: Instant::now() + Self::election_timeout(), heartbeat: Instant::now(),
            inflight: vec![false; members], generation: 0, failed: false,
        }));
        let handle = Self { inner, endpoint, gate: Arc::new(AsyncMutex::new(())) };
        handle.restore().await.expect("failed to restore Raft; refusing to vote");
        {
            let _guard = handle.gate.lock().await;
            handle.finish().await.expect("initialize storage");
        }
        handle.start_rpc_server();
        let this = handle.clone();
        task::spawn(async move {
            loop {
                sleep(Duration::from_millis(10)).await;
                this.tick().await;
            }
        });
        (handle, recver)
    }
    fn election_timeout() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(300..550))
    }
    pub fn term(&self) -> u64 { self.inner.lock().unwrap().published.term }
    pub fn is_leader(&self) -> bool {
        let r = self.inner.lock().unwrap();
        !r.failed && r.published.is_leader()
    }
    pub fn leader_hint(&self) -> usize { self.inner.lock().unwrap().core.leader }
    pub fn state_size(&self) -> usize {
        let r = self.inner.lock().unwrap();
        bincode::serialized_size(&r.core.durable()).unwrap_or(u64::MAX).min(usize::MAX as u64) as usize
    }
    pub async fn start(&self, cmd: &[u8]) -> Result<Start> {
        let result;
        {
            let _guard = self.gate.lock().await;
            {
                let mut r = self.inner.lock().unwrap();
                if r.failed { return Err(invalid("Raft stopped after storage failure").into()); }
                match r.core.propose(cmd) {
                    Some((index, term)) => result = Start { index, term },
                    None if r.core.state.is_leader() => return Err(invalid("Raft index exhausted").into()),
                    None => return Err(Error::NotLeader(r.core.leader)),
                }
            }
            self.finish().await?;
        }
        self.replicate();
        Ok(result)
    }
    pub async fn snapshot(&self, index: u64, snapshot: &[u8]) -> Result<()> {
        let _guard = self.gate.lock().await;
        {
            let mut r = self.inner.lock().unwrap();
            if r.failed { return Err(invalid("Raft stopped").into()); }
            r.core.compact(index, snapshot);
        }
        self.finish().await?;
        Ok(())
    }
    pub async fn cond_install_snapshot(&self, term: u64, index: u64, snapshot: &[u8]) -> bool {
        let _guard = self.gate.lock().await;
        let accepted = {
            let mut r = self.inner.lock().unwrap();
            !r.failed && r.core.acknowledge_snapshot(term, index, snapshot)
        };
        if accepted && self.finish().await.is_err() { return false; }
        accepted
    }
    // Requires the gate. No dependent output is published before this returns.
    async fn finish(&self) -> io::Result<()> {
        if let Err(e) = self.persist().await {
            self.inner.lock().unwrap().failed = true;
            return Err(e);
        }
        let mut r = self.inner.lock().unwrap();
        if r.failed { return Err(invalid("Raft stopped")); }
        r.published = r.core.state;
        if r.core.reset_timer {
            r.deadline = Instant::now() + Self::election_timeout();
            r.core.reset_timer = false;
        }
        for effect in r.core.take_applied() {
            let msg = match effect {
                Applied::Command { index, data } => ApplyMsg::Command { index, data },
                Applied::Snapshot { index, term, data } => ApplyMsg::Snapshot { index, term, data },
            };
            let _ = r.apply_ch.unbounded_send(msg);
        }
        Ok(())
    }
    async fn persist(&self) -> io::Result<()> {
        let image = {
            let mut r = self.inner.lock().unwrap();
            if r.failed { return Err(invalid("Raft stopped")); }
            if !r.core.dirty { return Ok(()); }
            let generation = storage::next_generation(r.generation).ok_or_else(|| invalid("storage generation exhausted"))?;
            r.generation = generation;
            Image { state: Persist { generation, peers: r.peers.clone(), me: r.me, durable: r.core.durable() },
                snapshot: r.core.snapshot.clone() }
        };
        // The previous primary pair remains usable while backup is truncated.
        // Once the new backup write completes it is the recovery authority while
        // either primary file is being replaced. Never expose outputs in between.
        let backup = bincode::serialize(&image).map_err(|_| invalid("encode image"))?;
        write_file("raft.backup", &backup).await?;
        let (state, snapshot) = encode_projections(&image)?;
        write_file("state", &state).await?;
        write_file("snapshot", &snapshot).await?;
        self.inner.lock().unwrap().core.dirty = false;
        Ok(())
    }
    async fn restore(&self) -> io::Result<()> {
        let backup_bytes = read_optional("raft.backup").await?;
        let state_bytes = read_optional("state").await?;
        let snapshot_bytes = read_optional("snapshot").await?;
        let backup: Option<Image> = match backup_bytes.as_ref().filter(|b| !b.is_empty()) {
            Some(bytes) => Some(bincode::deserialize(bytes).map_err(|_| invalid("incomplete backup image; fail closed"))?),
            None => None,
        };
        let mut primary: Option<Image> = None;
        if let (Some(state), Some(snapshot)) = (&state_bytes, &snapshot_bytes) {
            if let Ok(p) = bincode::deserialize::<Persist>(state) {
                let decoded = bincode::deserialize::<SnapshotImage>(snapshot).ok();
                if storage::primary_coherent(p.durable.base, p.generation,
                    decoded.as_ref().map(|s| s.generation), snapshot.is_empty()) {
                    let data = if p.durable.base == 0 { vec![] } else { decoded.unwrap().data };
                    primary = Some(Image { state: p, snapshot: data });
                }
            }
        }
        let primary_generation = primary.as_ref().map(|p| p.state.generation);
        let backup_generation = backup.as_ref().map(|p| p.state.generation);
        let image = match storage::select(primary_generation, backup_generation) {
            storage::Source::Primary => primary.unwrap(),
            storage::Source::Backup => backup.unwrap(),
            storage::Source::Absent => {
                if state_bytes.is_some() || snapshot_bytes.is_some() || backup_bytes.as_ref().map_or(false, |b| !b.is_empty()) {
                    return Err(invalid("no coherent durable image; refusing to reset a vote"));
                }
                return Ok(());
            }
        };
        let (state, snapshot) = encode_projections(&image)?;
        let repair_backup = if storage::backup_required(image.state.generation, backup_generation) {
            Some(bincode::serialize(&image).map_err(|_| invalid("encode recovery backup"))?)
        } else { None };
        {
            let mut r = self.inner.lock().unwrap();
            if image.state.peers != r.peers || image.state.me != r.me { return Err(invalid("Raft identity changed")); }
            if !r.core.restore(image.state.durable, image.snapshot) { return Err(invalid("invalid durable Raft state")); }
            r.generation = image.state.generation;
        }
        // If the primary was selected, it protects this backup replacement. If
        // the backup was the sole authority, preserve it until BOTH primary
        // projections have been repaired. This remains safe across another crash.
        if let Some(backup) = repair_backup { write_file("raft.backup", &backup).await?; }
        write_file("state", &state).await?;
        write_file("snapshot", &snapshot).await?;
        Ok(())
    }
    fn start_rpc_server(&self) {
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: Vote| {
            let this = this.clone(); async move { this.request_vote(args).await }
        });
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: Append| {
            let this = this.clone(); async move { this.append(args).await }
        });
        let this = self.clone();
        self.endpoint.add_rpc_handler(move |args: Install| {
            let this = this.clone(); async move { this.install(args).await }
        });
    }
    async fn request_vote(&self, args: Vote) -> VoteReply {
        let _guard = self.gate.lock().await;
        if self.inner.lock().unwrap().failed { return futures::future::pending().await; }
        let reply = self.inner.lock().unwrap().core.request_vote(args);
        if self.finish().await.is_err() { return futures::future::pending().await; }
        reply
    }
    async fn append(&self, args: Append) -> AppendReply {
        let _guard = self.gate.lock().await;
        if self.inner.lock().unwrap().failed { return futures::future::pending().await; }
        let reply = self.inner.lock().unwrap().core.append(args);
        if self.finish().await.is_err() { return futures::future::pending().await; }
        reply
    }
    async fn install(&self, args: Install) -> InstallReply {
        let _guard = self.gate.lock().await;
        if self.inner.lock().unwrap().failed { return futures::future::pending().await; }
        let reply = self.inner.lock().unwrap().core.install(args);
        if self.finish().await.is_err() { return futures::future::pending().await; }
        reply
    }
    async fn tick(&self) {
        let campaign;
        let replicate;
        {
            let _guard = self.gate.lock().await;
            {
                let mut r = self.inner.lock().unwrap();
                if r.failed { return; }
                campaign = if !r.core.state.is_leader() && Instant::now() >= r.deadline { r.core.election() } else { None };
                replicate = r.core.state.is_leader() && r.heartbeat <= Instant::now();
                if replicate { r.heartbeat = Instant::now() + Duration::from_millis(50); }
            }
            if self.finish().await.is_err() { return; }
        }
        if let Some(v) = campaign { self.send_votes(v); }
        if replicate { self.replicate(); }
    }
    fn send_votes(&self, request: Vote) {
        let peers = self.inner.lock().unwrap().peers.clone();
        for (peer, addr) in peers.into_iter().enumerate() {
            if peer == request.candidate { continue; }
            let this = self.clone();
            let request = request.clone();
            task::spawn(async move {
                let sent = request.term;
                if let Ok(reply) = this.endpoint.call_timeout(addr, request, Duration::from_millis(250)).await {
                    let _guard = this.gate.lock().await;
                    {
                        let mut r = this.inner.lock().unwrap();
                        if r.failed { return; }
                        r.core.vote_reply(peer, sent, reply);
                    }
                    if this.finish().await.is_ok() { this.replicate(); }
                }
            });
        }
    }
    fn replicate(&self) {
        let mut r = self.inner.lock().unwrap();
        if r.failed { return; }
        for peer in 0..r.peers.len() {
            if r.inflight[peer] { continue; }
            if let Some(request) = r.core.replication(peer) {
                r.inflight[peer] = true;
                let addr = r.peers[peer];
                let this = self.clone();
                task::spawn(async move {
                    match request {
                        Replication::Append(a) => {
                            let (sent, prev, end) = (a.term, a.prev, a.prev + a.entries.len() as u64);
                            let result = this.endpoint.call_timeout(addr, a, Duration::from_millis(200)).await;
                            let _guard = this.gate.lock().await;
                            {
                                let mut r = this.inner.lock().unwrap();
                                r.inflight[peer] = false;
                                if r.failed { return; }
                                if let Ok(reply) = result { r.core.append_reply(peer, sent, prev, end, reply); }
                            }
                            let _ = this.finish().await;
                        }
                        Replication::Snapshot(s) => {
                            let (sent, index) = (s.term, s.index);
                            let result = this.endpoint.call_timeout(addr, s, Duration::from_millis(300)).await;
                            let _guard = this.gate.lock().await;
                            {
                                let mut r = this.inner.lock().unwrap();
                                r.inflight[peer] = false;
                                if r.failed { return; }
                                if let Ok(reply) = result { r.core.install_reply(peer, sent, index, reply); }
                            }
                            let _ = this.finish().await;
                        }
                    }
                });
            }
        }
    }
}
impl net::rpc::Request for Vote { type Response = VoteReply; const ID: u64 = 1; }
impl net::rpc::Request for Append { type Response = AppendReply; const ID: u64 = 5; }
impl net::rpc::Request for Install { type Response = InstallReply; const ID: u64 = 6; }

#[cfg(test)]
mod regressions {
    use super::*;
    fn image(generation: u64, base: u64) -> Image {
        Image {
            state: Persist { generation, peers: vec!["127.0.0.1:1".parse().unwrap()], me: 0,
                durable: Durable { term: 9, voted_for: Some(0), base, base_term: 8,
                    log: vec![Entry { term: 9, data: b"retained command".to_vec() }] } },
            snapshot: vec![base as u8; 7],
        }
    }
    async fn host() -> RaftHandle {
        let endpoint = net::Endpoint::bind("127.0.0.1:1").await.unwrap();
        let (apply_ch, _) = mpsc::unbounded();
        RaftHandle { endpoint, gate: Arc::new(AsyncMutex::new(())), inner: Arc::new(Mutex::new(Raft {
            peers: vec!["127.0.0.1:1".parse().unwrap()], me: 0, apply_ch,
            core: Core::new(0, 1), published: State::initial(), deadline: Instant::now(), heartbeat: Instant::now(),
            inflight: vec![false], generation: 0, failed: false,
        })) }
    }
    // Model a crash after each completed host operation of persist. These are
    // real FsSim files decoded by the actual recovery path, with changed
    // snapshot boundaries and retained command bytes in the new generation.
    #[madsim::test]
    async fn recovery_at_every_image_write_boundary() {
        let host = host().await;
        let before = image(10, 2);
        let after = image(11, 3);
        let old_backup = bincode::serialize(&before).unwrap();
        let new_backup = bincode::serialize(&after).unwrap();
        let old_state = bincode::serialize(&before.state).unwrap();
        let new_state = bincode::serialize(&after.state).unwrap();
        let old_snapshot = bincode::serialize(&SnapshotImage { generation: 10, data: before.snapshot.clone() }).unwrap();
        let new_snapshot = bincode::serialize(&SnapshotImage { generation: 11, data: after.snapshot.clone() }).unwrap();
        for phase in 0..7 {
            write_file("raft.backup", if phase == 0 { &old_backup } else if phase == 1 { &[] } else { &new_backup }).await.unwrap();
            write_file("state", if phase < 3 { &old_state } else if phase == 3 { &[] } else { &new_state }).await.unwrap();
            write_file("snapshot", if phase < 5 { &old_snapshot } else if phase == 5 { &[] } else { &new_snapshot }).await.unwrap();
            {
                let mut r = host.inner.lock().unwrap();
                r.core = Core::new(0, 1);
                r.generation = 0;
            }
            host.restore().await.unwrap();
            let r = host.inner.lock().unwrap();
            let expected = if phase <= 1 { &before } else { &after };
            assert_eq!(r.generation, expected.state.generation, "phase {phase}");
            assert_eq!(r.core.state.term, 9);
            assert_eq!(r.core.state.voted_for, Some(0), "vote lost at phase {phase}");
            assert_eq!(r.core.base, expected.state.durable.base);
            assert_eq!(r.core.log, expected.state.durable.log);
            assert_eq!(r.core.snapshot, expected.snapshot);
        }
    }
    #[madsim::test]
    async fn recovery_repairs_primary_before_replacing_sole_backup() {
        let host = host().await;
        let old = image(10, 2);
        let current = image(11, 3);
        write_file("raft.backup", &bincode::serialize(&current).unwrap()).await.unwrap();
        write_file("state", &bincode::serialize(&current.state).unwrap()).await.unwrap();
        write_file("snapshot", &bincode::serialize(&SnapshotImage { generation: 10, data: old.snapshot }).unwrap()).await.unwrap();
        host.restore().await.unwrap();
        // A second crash occurs at the next backup truncate. Recovery must have
        // repaired the primary first, so it remains an independent authority.
        write_file("raft.backup", &[]).await.unwrap();
        host.inner.lock().unwrap().core = Core::new(0, 1);
        host.restore().await.expect("a second crash must not erase the sole coherent image");
        let r = host.inner.lock().unwrap();
        assert_eq!(r.core.base, 3);
        assert_eq!(r.core.snapshot, current.snapshot);
        assert_eq!(r.core.state.voted_for, Some(0));
    }

    #[madsim::test]
    async fn incoherent_recovery_fails_closed() {
        let host = host().await;
        let before = image(10, 2);
        write_file("state", &bincode::serialize(&before.state).unwrap()).await.unwrap();
        write_file("snapshot", &[]).await.unwrap();
        write_file("raft.backup", &[]).await.unwrap();
        assert!(host.restore().await.is_err(), "must not forget an unknown durable vote");
    }
}
