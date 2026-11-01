"""Only declarations, unfinished student entries and non-protocol RPC wiring."""
from pathlib import Path
import shutil

def prepare(root):
    root = Path(root)
    task = root / 'tasks/election_start'
    for source, destination in [('core.rs', 'src/raft/vote_core.rs'), ('lemmas.rs', 'proof/vote_lemmas.rs')]:
        dest = root / 'madraft' / destination
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(task / source, dest)
    file = root / 'madraft/src/raft/raft.rs'
    text = file.read_text()
    def replace(old, new):
        nonlocal text
        if text.count(old) != 1:
            raise ValueError('Election-start scaffold changed; review preparation: ' + old[:70])
        text = text.replace(old,new)
    replace('    inner: Arc<Mutex<Raft>>,', '''    inner: Arc<Mutex<Raft>>,
    vote_serial: Arc<futures::lock::Mutex<()>>,
    vote_hooks: Arc<Mutex<vote_host::Hooks>>,''')
    replace('    state: State,', '    vote: vote_core::Core,')
    replace('use state::{State, Role};', '''// The old State type remains an unused student source, not a second authority.
#[path = "../../../tasks/election_start/api.rs"] mod vote_api;
#[path = "vote_core.rs"] mod vote_core;
#[path = "../../../tasks/election_start/host.rs"] mod vote_host;
#[cfg(test)]
#[path = "../../../tasks/election_start/rpc_tests.rs"] mod vote_tests;''')
    replace('''    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        let endpoint''', '''    pub async fn new(peers: Vec<SocketAddr>, me: usize) -> (Self, MsgRecver) {
        Self::new_receiver(peers, me, false).await
    }

    // Same builder and registered RPC on restart; only file existence is stricter.
    async fn new_receiver(peers: Vec<SocketAddr>, me: usize, recovery_only: bool) -> (Self, MsgRecver) {
        let vote = vote_host::load(vote_api::Config{node:me as u64,members:peers.len() as u64}, recovery_only)
            .await.expect("restore receiver from actual file");
        let endpoint''')
    replace('            state: State::default(),', '            vote,')
    replace('        let handle = RaftHandle { inner, endpoint };', '''        let handle = RaftHandle { inner, endpoint,
            vote_serial: Arc::new(futures::lock::Mutex::new(())),
            vote_hooks: Arc::new(Mutex::new(vote_host::Hooks::default())) };''')
    replace('        handle.restore().await.expect("failed to restore");', '        // Receiver recovery already called the actual Core above. Other labs remain TODO.')
    replace('        raft.state.term', '        raft.vote.inspect().state.term')
    replace('        raft.state.is_leader()', '        raft.vote.inspect().role == vote_api::Role::Leader')
    replace('!self.state.is_leader()', 'self.vote.inspect().role != vote_api::Role::Leader')
    replace('async move { this.request_vote(args).await.unwrap() }', 'async move { this.request_vote(args).await.map_err(|e| e.to_string()) }')
    start = text.index('    async fn request_vote(&self,')
    end = text.index('\n}\n',start)
    text = text[:start] + '''    async fn request_vote(&self, args: RequestVoteArgs) -> Result<RequestVoteReply> {
        // Public TCB: serialize a complete receive/save/complete operation.
        let _serial = self.vote_serial.lock().await;
        if let Some(observer) = &self.vote_hooks.lock().unwrap().incoming {
            let _ = observer.try_send(vote_api::Request{term:args.term,candidate:args.candidate_id,
                log:vote_api::Log{term:args.last_log_term,index:args.last_log_index}});
        }
        let decision = { self.inner.lock().unwrap().request_vote(args) };
        let write = match decision {
            vote_api::Receive::Save(w) => w,
            vote_api::Receive::Reject(f) => return Err(io::Error::other(format!("receiver: {f:?}")).into()),
        };
        let hooks = self.vote_hooks.lock().unwrap().clone();
        let saved = vote_host::save(write, &hooks).await;
        let reply = { self.inner.lock().unwrap().vote.complete(vote_api::Completion {
            op:write.record.op, success:saved.is_ok() }) };
        saved?;
        let Some(vote_api::Action::Reply(reply)) = reply else {
            return Err(io::Error::other("core did not authorize receiver reply").into());
        };
        Ok(RequestVoteReply { term:reply.term, vote_granted:reply.granted })
    }''' + text[end:]
    replace('''    fn request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        todo!("handle RequestVote RPC");
    }''', '''    fn request_vote(&mut self, args: RequestVoteArgs) -> vote_api::Receive {
        self.vote.receive(vote_api::Request { term:args.term, candidate:args.candidate_id,
            log:vote_api::Log{term:args.last_log_term,index:args.last_log_index} })
    }''')
    replace('''struct RequestVoteArgs {
    // Your data here.
}''', '''struct RequestVoteArgs {
    term: u64, candidate_id: u64, last_log_term: u64, last_log_index: u64,
}''')
    replace('''struct RequestVoteReply {
    // Your data here.
}''', '''struct RequestVoteReply { term: u64, vote_granted: bool }''')
    replace('    type Response = RequestVoteReply;', '    type Response = std::result::Result<RequestVoteReply, String>;')
    # Remove the unused old sender example, which reads no authoritative Core.
    start = text.index('    // Here is an example to send RPC and manage concurrent tasks.')
    end = text.index('\n}\n',start)
    text = text[:start] + '    // Reply counting and leader promotion remain OPEN.\n' + text[end:]
    methods = (task / 'driver.rs').read_text()
    insert = text.index('    /// Start agreement on the next command')
    text = text[:insert] + methods + '\n' + text[insert:]
    file.write_text(text)
    shutil.copy2(task / 'AGENTS.md', root / 'AGENTS.md')
    return ['madraft/src/raft/vote_core.rs', 'madraft/proof/vote_lemmas.rs']
