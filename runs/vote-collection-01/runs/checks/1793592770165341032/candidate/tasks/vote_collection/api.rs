//! Fixed receiver/campaign/collection SPECIFICATIONS. No protocol implementation.
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config { pub node: u64, pub members: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Log { pub term: u64, pub index: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Persistent { pub term: u64, pub voted_for: Option<u64>, pub log: Log }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role { Follower, Candidate, Leader }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Initial { pub state: Persistent, pub role: Role }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request { pub term: u64, pub candidate: u64, pub log: Log }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply { pub node: u64, pub term: u64, pub candidate: u64, pub granted: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record { pub op: u64, pub state: Persistent }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Send { pub config: Config, pub request: Request }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action { Reply(Reply), Send(Send), Persisted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Write { pub record: Record, pub request: Option<Request>, pub action: Action }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Completion { pub op: u64, pub success: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault { Invalid, Busy, Exhausted, Halted, Leader, TermExhausted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receive { Save(Write), Reject(Fault) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Campaign { Save(Write), Reject(Fault) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice { pub node: u64, pub term: u64, pub candidate: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View { pub config: Config, pub state: Persistent, pub role: Role,
    pub next: u64, pub pending: Option<Write>, pub halted: bool,
    pub round: Option<u64>, pub self_saved: bool }
#[derive(Clone, Copy, Debug)]
pub enum Event { Start, Receive(Request), Complete(Completion), Rpc(RpcContext, RpcResult) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RpcContext { pub sent: Send, pub peer: u64 }
// Actual wire fields only. Identity comes from the ORIGINAL call context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcResult { Reply { term: u64, granted: bool }, Error }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collect { Save(Write), Counted, Ignored, Reject(Fault) }

pub open spec fn valid_config(c: Config) -> bool { c.members > 0 && c.node < c.members }
pub open spec fn valid_log(l: Log) -> bool { (l.index == 0) == (l.term == 0) }
pub open spec fn valid_state(c: Config, s: Persistent) -> bool {
    valid_config(c) && valid_log(s.log) && s.log.term <= s.term
    && (s.voted_for.is_some() ==> s.term > 0 && s.voted_for.unwrap() < c.members)
}
pub open spec fn valid_initial(c: Config, s: Initial) -> bool {
    valid_state(c, s.state) && (s.state.term == 0 ==> s.role == Role::Follower)
}
pub open spec fn valid_request(c: Config, r: Request) -> bool {
    r.term > 0 && r.candidate < c.members && valid_log(r.log) && r.log.term <= r.term
}
pub open spec fn valid_record(c: Config, r: Record) -> bool {
    valid_state(c, r.state) && r.op < u64::MAX
}
pub open spec fn fresh_view(c: Config, s: Initial) -> View {
    View { config: c, state: s.state, role: s.role, next: 0, pending: None, halted: false, round: None, self_saved: false }
}
pub open spec fn recovered_view(c: Config, r: Record) -> View {
    View { config: c, state: r.state, role: Role::Follower,
        next: (r.op + 1) as u64, pending: None, halted: false, round: None, self_saved: false }
}
pub open spec fn up_to_date(remote: Log, local: Log) -> bool {
    remote.term > local.term || (remote.term == local.term && remote.index >= local.index)
}
pub open spec fn grants(v: View, r: Request) -> bool {
    r.term >= v.state.term && up_to_date(r.log, v.state.log)
    && (r.term > v.state.term || v.state.voted_for.is_none() || v.state.voted_for == Some(r.candidate))
}
pub open spec fn decided(v: View, r: Request) -> Persistent {
    Persistent { term: if r.term > v.state.term { r.term } else { v.state.term },
        voted_for: if grants(v,r) { Some(r.candidate) }
            else if r.term > v.state.term { None } else { v.state.voted_for }, log: v.state.log }
}
// This bounded interface saves every accepted decision, including unchanged
// rejections. It trades extra I/O for one explicit authorization boundary.
pub open spec fn receive_result(v: View, r: Request) -> Receive {
    if v.halted { Receive::Reject(Fault::Halted) }
    else if v.pending.is_some() { Receive::Reject(Fault::Busy) }
    else if !valid_request(v.config,r) { Receive::Reject(Fault::Invalid) }
    else if v.next == u64::MAX { Receive::Reject(Fault::Exhausted) }
    else { Receive::Save(Write { record: Record { op: v.next, state: decided(v,r) }, request: Some(r),
        action: Action::Reply(Reply { node: v.config.node, term: decided(v,r).term, candidate: r.candidate, granted: grants(v,r) }) }) }
}
pub open spec fn receive_after(v: View, r: Request, result: Receive) -> View {
    match result { Receive::Reject(Fault::Exhausted) => if r.term > v.state.term {
        View{state:Persistent{term:r.term,voted_for:None,log:v.state.log},role:Role::Follower,
            round:None,self_saved:false,halted:true,..v}
        } else { v }, Receive::Reject(_) => v, Receive::Save(w) => View {
        config: v.config, state: w.record.state,
        role: if r.term > v.state.term { Role::Follower } else { v.role },
        next: (v.next + 1) as u64, pending: Some(w), halted: false,
        round: if r.term > v.state.term { None } else { v.round },
        self_saved: v.self_saved && r.term <= v.state.term } }
}
pub open spec fn matches(v: View, n: Completion) -> bool {
    v.pending.is_some() && v.pending.unwrap().record.op == n.op
}
pub open spec fn complete_result(v: View, n: Completion) -> Option<Action> {
    if matches(v,n) && n.success { Some(v.pending.unwrap().action) } else { None }
}
pub open spec fn complete_after(v: View, n: Completion) -> View {
    View { pending: if matches(v,n) { None } else { v.pending },
        halted: v.halted || (matches(v,n) && !n.success),
        round: if matches(v,n) && !n.success { None } else { v.round },
        self_saved: if matches(v,n) { n.success && (v.self_saved || v.pending.unwrap().action is Send) }
            else { v.self_saved }, ..v }
}
// A prepared Send inside Write is data, NOT permission to use the network.
// Only the actual complete return authorizes an Action::Send.
pub open spec fn start_result(v: View) -> Campaign {
    if v.halted { Campaign::Reject(Fault::Halted) }
    else if v.pending.is_some() { Campaign::Reject(Fault::Busy) }
    else if v.role == Role::Leader { Campaign::Reject(Fault::Leader) }
    else if v.state.term == u64::MAX { Campaign::Reject(Fault::TermExhausted) }
    else if v.next == u64::MAX { Campaign::Reject(Fault::Exhausted) }
    else {
        let state = Persistent{term:(v.state.term + 1) as u64, voted_for:Some(v.config.node), log:v.state.log};
        let request = Request{term:state.term,candidate:v.config.node,log:v.state.log};
        Campaign::Save(Write{record:Record{op:v.next,state},request:Some(request),
            action:Action::Send(Send{config:v.config,request})})
    }
}
pub open spec fn start_after(v: View, result: Campaign) -> View {
    match result { Campaign::Reject(_) => v, Campaign::Save(w) => View {
        config:v.config,state:w.record.state,role:Role::Candidate,next:(v.next + 1) as u64,
        pending:Some(w),halted:false,round:Some(w.record.state.term),self_saved:false } }
}
pub open spec fn choice(v: View) -> Choice {
    Choice{node:v.config.node,term:v.state.term,candidate:v.state.voted_for.unwrap()}
}
pub open spec fn start_choices(h: Seq<Choice>, v: View, result: Campaign) -> Seq<Choice> {
    match result { Campaign::Save(_) => h.push(choice(start_after(v,result))), Campaign::Reject(_) => h }
}
pub open spec fn append_action(h: Seq<Choice>, out: Option<Action>) -> Seq<Choice> {
    match out { Some(Action::Reply(r)) => if r.granted {
        h.push(Choice{node:r.node,term:r.term,candidate:r.candidate}) } else { h }, _ => h }
}
pub open spec fn unique(h: Seq<Choice>) -> bool {
    forall|i:int,j:int| 0 <= i < h.len() && 0 <= j < h.len()
        && (#[trigger] h[i]).node == (#[trigger] h[j]).node && h[i].term == h[j].term
        ==> h[i].candidate == h[j].candidate
}
// The set representation is not fixed. Actual queries must match this set.
pub open spec fn voters_domain(v: View, votes: Set<u64>) -> bool {
    votes.finite() && (forall|n:u64| #[trigger] votes.contains(n) ==> n < v.config.members)
        && votes.len() <= v.config.members
        && ((!v.self_saved || v.halted || v.round.is_none() || v.role != Role::Candidate) ==> votes == Set::<u64>::empty())
}
pub open spec fn active(v: View) -> bool {
    !v.halted && v.pending.is_none() && v.role == Role::Candidate
        && v.self_saved && v.round == Some(v.state.term)
}
pub open spec fn receive_votes(v: View, votes: Set<u64>, r: Request, out: Receive) -> Set<u64> {
    if (out is Save || out == Receive::Reject(Fault::Exhausted)) && r.term > v.state.term { Set::empty() } else { votes }
}
pub open spec fn start_votes(votes: Set<u64>, out: Campaign) -> Set<u64> {
    if out is Save { Set::empty() } else { votes }
}
pub open spec fn completion_votes(v: View, votes: Set<u64>, n: Completion) -> Set<u64> {
    if matches(v,n) {
        if !n.success { Set::empty() }
        else if v.pending.unwrap().action is Send { Set::<u64>::empty().insert(v.config.node) }
        else { votes }
    } else { votes }
}
pub open spec fn valid_context(v: View, c: RpcContext) -> bool {
    c.sent.config == v.config && c.peer < v.config.members && c.peer != v.config.node
        && c.sent.request.candidate == v.config.node && valid_request(v.config,c.sent.request)
}
pub open spec fn higher(v: View, c: RpcContext, r: RpcResult) -> bool {
    !v.halted && v.pending.is_none() && valid_context(v,c)
        && r is Reply && r->Reply_term > v.state.term
}
pub open spec fn collect_result(v: View, votes: Set<u64>, c: RpcContext, r: RpcResult) -> Collect {
    if v.halted { Collect::Reject(Fault::Halted) }
    else if v.pending.is_some() { Collect::Reject(Fault::Busy) }
    else if !valid_context(v,c) { Collect::Reject(Fault::Invalid) }
    else { match r {
        RpcResult::Error => Collect::Ignored,
        RpcResult::Reply{term,granted} => {
            if term > v.state.term {
                if v.next == u64::MAX { Collect::Reject(Fault::Exhausted) }
                else { Collect::Save(Write{record:Record{op:v.next,state:Persistent{
                    term,voted_for:None,log:v.state.log}},request:None,action:Action::Persisted}) }
            } else if active(v) && granted && term == v.state.term && c.sent.request.term == term
                && c.sent.request.log == v.state.log && !votes.contains(c.peer) { Collect::Counted }
            else { Collect::Ignored }
        }
    } }
}
pub open spec fn collect_after(v: View, c: RpcContext, r: RpcResult, out: Collect) -> View {
    if higher(v,c,r) {
        View{state:Persistent{term:r->Reply_term,voted_for:None,log:v.state.log},role:Role::Follower,
            next:if v.next < u64::MAX { (v.next + 1) as u64 } else { v.next },
            pending:match out { Collect::Save(w) => Some(w), _ => None },
            halted:v.next == u64::MAX,round:None,self_saved:false,..v}
    } else { v }
}
pub open spec fn collect_votes(v: View, votes: Set<u64>, c: RpcContext, r: RpcResult, out: Collect) -> Set<u64> {
    if higher(v,c,r) { Set::empty() }
    else if out == Collect::Counted { votes.insert(c.peer) } else { votes }
}
// Nonvacuity and a specification sanity check; neither proves an implementation.
pub proof fn rules_are_nonempty()
    ensures grants(fresh_view(Config{node:0,members:3}, Initial{
        state:Persistent{term:2,voted_for:None,log:Log{term:1,index:4}},role:Role::Follower}),
        Request{term:2,candidate:1,log:Log{term:1,index:4}}),
{ }
}
