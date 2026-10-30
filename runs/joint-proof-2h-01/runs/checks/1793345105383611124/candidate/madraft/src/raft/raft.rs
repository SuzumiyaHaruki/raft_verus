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
    storage_gate: Arc<futures::lock::Mutex<()>>,
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
}

#[path = "state.rs"]
mod state;
use state::{State, VoteRequest as RequestVoteArgs, VoteReply as RequestVoteReply};

/// Data needs to be persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Persist {
    version: u64,
    term: u64,
    voted_for: Option<usize>,
    me: usize,
    members: usize,
}

impl fmt::Debug for Raft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Raft({})", self.me)
    }
}

// HINT: put async functions here
impl RaftHandle {
    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        let state = State::new(me, peers.len()).expect("invalid Raft membership");
        let endpoint = net::Endpoint::bind(peers[me]).await.expect("bind Raft RPC");
        let (apply_ch, recver) = mpsc::unbounded();
        let inner = Arc::new(Mutex::new(Raft {
            endpoint: endpoint.clone(),
            peers,
            me,
            apply_ch,
            state,
        }));
        let handle = RaftHandle { inner, endpoint, storage_gate: Arc::new(futures::lock::Mutex::new(())) };
        // initialize from state persisted before a crash
        handle.restore().await.expect("failed to restore");
        handle.start_rpc_server();

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
    // Caller holds storage_gate from decision through reply release.
    // Until log/snapshot support is implemented this image contains votes only.
    async fn persist(&self) -> io::Result<()> {
        let persist = {
            let raft = self.inner.lock().unwrap();
            Persist { version: 1, term: raft.state.term, voted_for: raft.state.voted_for,
                me: raft.state.me, members: raft.state.members }
        };
        let bytes = bincode::serialize(&persist)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        // A crash after truncation leaves a non-decodable image and recovery
        // stops. It must never turn that image into permission to vote again.
        let file = fs::File::create("state").await?;
        file.write_all_at(&bytes, 0).await?;
        file.sync_all().await?;
        Ok(())
    }

    async fn restore(&self) -> io::Result<()> {
        // No snapshot can yet be interpreted by this partial implementation.
        match fs::read("snapshot").await {
            Ok(bytes) if !bytes.is_empty() => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "snapshot recovery unfinished"));
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        match fs::read("state").await {
            Ok(bytes) => {
                let saved: Persist = bincode::deserialize(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let mut raft = self.inner.lock().unwrap();
                if saved.version != 1 || saved.me != raft.me || saved.members != raft.peers.len() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "inconsistent vote image"));
                }
                raft.state = State::restore_vote(saved.me, saved.members, saved.term, saved.voted_for)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid durable vote"))?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn start_rpc_server(&self) {
        let net = self.endpoint.clone();

        let this = self.clone();
        net.add_rpc_handler(move |args: RequestVoteArgs| {
            let this = this.clone();
            async move { this.request_vote(args).await.unwrap() }
        });
        // add more RPC handers here
    }

    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        let _operation = self.storage_gate.lock().await;
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
        self.state.request_vote(args)
    }

    // Here is an example to generate random number.
    fn generate_election_timeout() -> Duration {
        // see rand crate for more details
        Duration::from_millis(rand::thread_rng().gen_range(150..300))
    }

    // Here is an example to send RPC and manage concurrent tasks.
    fn send_vote_request(&mut self) {
        let args: RequestVoteArgs = todo!("construct RPC request");
        let timeout = Self::generate_election_timeout();
        let net = self.endpoint.clone();

        let mut rpcs = FuturesUnordered::new();
        for (i, &peer) in self.peers.iter().enumerate() {
            if i == self.me {
                continue;
            }
            // NOTE: `call` function takes ownerships
            let net = net.clone();
            let args = args.clone();
            rpcs.push(async move {
                net.call_timeout::<RequestVoteArgs>(peer, args, timeout)
                    .await
            });
        }

        // spawn a concurrent task
        task::spawn(async move {
            // handle RPC tasks in completion order
            while let Some(res) = rpcs.next().await {
                todo!("handle RPC results");
            }
        }); // MadSim 0.2 detaches JoinHandle on drop.
    }
}

impl net::rpc::Request for RequestVoteArgs {
    type Response = RequestVoteReply;
    const ID: u64 = 1;
}

#[cfg(test)]
mod vote_tests {
    use super::*;

    // Exercises the actual RPC-facing wrapper, storage and recovery path.
    #[madsim::test]
    async fn vote_persists_before_reply_and_restores() {
        let peers = vec!["127.0.0.1:20701".parse().unwrap(), "127.0.0.1:20702".parse().unwrap()];
        let (raft, _) = RaftHandle::new(peers, 0).await;
        let request = RequestVoteArgs { term: 7, candidate: 1, last_index: 0, last_term: 0 };
        let sender = net::Endpoint::bind("127.0.0.1:20702".parse::<SocketAddr>().unwrap()).await.unwrap();
        let reply = sender.call_timeout::<RequestVoteArgs>(
            "127.0.0.1:20701".parse().unwrap(), request, Duration::from_secs(1)).await.unwrap();
        assert!(reply.granted);
        let saved: Persist = bincode::deserialize(&fs::read("state").await.unwrap()).unwrap();
        assert_eq!(saved.term, 7);
        assert_eq!(saved.voted_for, Some(1));
        raft.inner.lock().unwrap().state = State::new(0, 2).unwrap();
        raft.restore().await.unwrap();
        assert!(!raft.request_vote(RequestVoteArgs { candidate: 0, ..request }).await.unwrap().granted);
        assert!(raft.request_vote(request).await.unwrap().granted);
        assert!(!raft.request_vote(RequestVoteArgs { term: 6, ..request }).await.unwrap().granted);
        // An interrupted truncate/write must stop recovery, never reset votes.
        let _truncated = fs::File::create("state").await.unwrap();
        assert!(raft.restore().await.is_err());
    }
}
