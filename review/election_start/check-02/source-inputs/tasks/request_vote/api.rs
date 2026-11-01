//! Fixed observational interface and SPECIFICATIONS, not a voting implementation.
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
pub struct Write { pub record: Record, pub request: Request, pub reply: Reply }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Completion { pub op: u64, pub success: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault { Invalid, Busy, Exhausted, Halted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receive { Save(Write), Reject(Fault) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View { pub config: Config, pub state: Persistent, pub role: Role,
    pub next: u64, pub pending: Option<Write>, pub halted: bool }
#[derive(Clone, Copy, Debug)]
pub enum Event { Receive(Request), Complete(Completion) }

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
    View { config: c, state: s.state, role: s.role, next: 0, pending: None, halted: false }
}
pub open spec fn recovered_view(c: Config, r: Record) -> View {
    View { config: c, state: r.state, role: Role::Follower,
        next: (r.op + 1) as u64, pending: None, halted: false }
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
    else { Receive::Save(Write { record: Record { op: v.next, state: decided(v,r) }, request: r,
        reply: Reply { node: v.config.node, term: decided(v,r).term, candidate: r.candidate, granted: grants(v,r) } }) }
}
pub open spec fn receive_after(v: View, r: Request, result: Receive) -> View {
    match result { Receive::Reject(_) => v, Receive::Save(w) => View {
        config: v.config, state: w.record.state,
        role: if r.term > v.state.term { Role::Follower } else { v.role },
        next: (v.next + 1) as u64, pending: Some(w), halted: false } }
}
pub open spec fn matches(v: View, n: Completion) -> bool {
    v.pending.is_some() && v.pending.unwrap().record.op == n.op
}
pub open spec fn complete_result(v: View, n: Completion) -> Option<Reply> {
    if matches(v,n) && n.success { Some(v.pending.unwrap().reply) } else { None }
}
pub open spec fn complete_after(v: View, n: Completion) -> View {
    View { pending: if matches(v,n) { None } else { v.pending },
        halted: v.halted || (matches(v,n) && !n.success), ..v }
}
pub open spec fn append_reply(h: Seq<Reply>, r: Option<Reply>) -> Seq<Reply> {
    match r { Some(a) => h.push(a), None => h }
}
pub open spec fn unique(h: Seq<Reply>) -> bool {
    forall|i:int, j:int| 0 <= i < h.len() && 0 <= j < h.len()
        && (#[trigger] h[i]).granted && (#[trigger] h[j]).granted
        && h[i].node == h[j].node && h[i].term == h[j].term ==> h[i].candidate == h[j].candidate
}
// Nonvacuity and a specification sanity check; neither proves an implementation.
pub proof fn rules_are_nonempty()
    ensures grants(fresh_view(Config{node:0,members:3}, Initial{
        state:Persistent{term:2,voted_for:None,log:Log{term:1,index:4}},role:Role::Follower}),
        Request{term:2,candidate:1,log:Log{term:1,index:4}}),
{ }
}
