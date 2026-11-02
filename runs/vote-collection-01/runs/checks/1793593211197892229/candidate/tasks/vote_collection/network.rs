//! Fixed no-restart execution/observation plumbing, not a Raft invariant.
//! Records actual outputs even if wrong. Only routing/bounds/pending-I/O guard delivery.
use crate::{vote_api::*, vote_core::Core};
use vstd::prelude::*;
use crate::vote_api::View as StateView;
verus! {
#[derive(Clone, Copy)]
pub struct Dispatch { pub send: usize, pub peer: u64 }
#[derive(Clone, Copy)]
pub struct Proposed { pub write: Write, pub request: Option<Dispatch> }
#[derive(Clone, Copy)]
pub struct Sent { pub node: u64, pub write: Write, pub notice: Completion, pub payload: Send }
#[derive(Clone, Copy)]
pub struct Returned { pub node: u64, pub request: Option<Dispatch>, pub payload: Reply }
#[derive(Clone, Copy)]
pub enum Step { Start(usize), Receive(Dispatch), Save{node:usize,success:bool}, Deliver(usize), Error(Dispatch) }
pub struct Observation { pub state: StateView, pub votes: Set<u64>, pub choices: Seq<Choice>, pub base: Option<Record> }
pub struct Trace {
    pub nodes: Seq<Observation>, pub pending: Seq<Option<Proposed>>, pub sent: Seq<Sent>,
    pub replies: Seq<Returned>, pub delivered: Seq<usize>,
}
pub struct Network {
    pub nodes: Vec<Core>, pub pending: Vec<Option<Proposed>>, pub sent: Vec<Sent>,
    pub replies: Vec<Returned>, pub delivered: Ghost<Seq<usize>>,
}
pub open spec fn observe(c: Core) -> Observation {
    Observation{state:c.view(),votes:c.votes(),choices:c.choices(),base:c.base()}
}
impl Network {
    pub open spec fn trace(&self) -> Trace {
        Trace{nodes:self.nodes@.map_values(|c:Core| observe(c)),pending:self.pending@,
            sent:self.sent@,replies:self.replies@,delivered:self.delivered@}
    }
}
pub open spec fn shape(t: Trace) -> bool {
    t.nodes.len() > 0 && t.nodes.len() == t.pending.len()
    && (forall|j:int| 0 <= j < t.sent.len() ==> (#[trigger] t.sent[j]).node < t.nodes.len())
    && forall|i:int| 0 <= i < t.nodes.len() ==> (#[trigger] t.nodes[i]).state.config == Config{node:i as u64,members:t.nodes.len() as u64}
}
pub open spec fn boot(t: Trace) -> bool {
    shape(t) && t.sent.len()==0 && t.replies.len()==0 && t.delivered.len()==0
    && forall|i:int| 0 <= i < t.nodes.len() ==> {
        let o=#[trigger] t.nodes[i];
        &&& valid_state(o.state.config,o.state.state)
        &&& o.state.next==0 && o.state.pending.is_none() && !o.state.halted
        &&& o.state.round.is_none() && !o.state.self_saved
        &&& o.votes==Set::<u64>::empty() && o.choices==Seq::<Choice>::empty() && o.base.is_none()
        &&& t.pending[i].is_none()
    }
}
pub open spec fn routed(t: Trace, d: Dispatch) -> bool {
    d.send < t.sent.len() && d.peer < t.nodes.len() && d.peer != t.sent[d.send as int].node
}
pub open spec fn context(t: Trace, d: Dispatch) -> RpcContext {
    RpcContext{sent:t.sent[d.send as int].payload,peer:d.peer}
}
pub open spec fn reply_result(r: Returned) -> RpcResult {
    RpcResult::Reply{term:r.payload.term,granted:r.payload.granted}
}
pub open spec fn enabled(t: Trace, s: Step) -> bool {
    match s {
        Step::Start(n) => n < t.nodes.len(),
        Step::Receive(d) | Step::Error(d) => routed(t,d),
        Step::Save{node,success:_} => node < t.nodes.len() && t.pending[node as int].is_some(),
        Step::Deliver(i) => i < t.replies.len() && t.replies[i as int].request.is_some()
            && routed(t,t.replies[i as int].request.unwrap()),
    }
}
pub open spec fn receiver(o: Observation, r: Request) -> Observation {
    let out=receive_result(o.state,r);
    Observation{state:receive_after(o.state,r,out),votes:receive_votes(o.state,o.votes,r,out),..o}
}
pub open spec fn starter(o: Observation) -> Observation {
    let out=start_result(o.state);
    Observation{state:start_after(o.state,out),votes:start_votes(o.votes,out),
        choices:start_choices(o.choices,o.state,out),..o}
}
pub open spec fn finisher(o: Observation, n: Completion) -> Observation {
    Observation{state:complete_after(o.state,n),votes:completion_votes(o.state,o.votes,n),
        choices:append_action(o.choices,complete_result(o.state,n)),..o}
}
pub open spec fn collector(o: Observation, c: RpcContext, r: RpcResult) -> Observation {
    let out=collect_result(o.state,o.votes,c,r);
    Observation{state:collect_after(o.state,c,r,out),votes:collect_votes(o.state,o.votes,c,r,out),..o}
}
pub open spec fn after_rpc(t: Trace, d: Dispatch, r: RpcResult) -> Trace {
    let c=context(t,d); let n=t.sent[d.send as int].node as int; let o=t.nodes[n];
    let out=collect_result(o.state,o.votes,c,r);
    Trace{nodes:t.nodes.update(n,collector(o,c,r)),pending:match out {
        Collect::Save(w) => t.pending.update(n,Some(Proposed{write:w,request:None})), _ => t.pending },..t}
}
// This is correspondence bookkeeping for ALL calls, never a safety filter.
pub open spec fn step_after(t: Trace, s: Step) -> Trace {
    match s {
        Step::Start(n) => {
            let o=t.nodes[n as int]; let out=start_result(o.state);
            Trace{nodes:t.nodes.update(n as int,starter(o)),pending:match out {
                Campaign::Save(w) => t.pending.update(n as int,Some(Proposed{write:w,request:None})), _ => t.pending },..t}
        },
        Step::Receive(d) => {
            let n=d.peer as int;let r=t.sent[d.send as int].payload.request;let o=t.nodes[n];
            Trace{nodes:t.nodes.update(n,receiver(o,r)),pending:match receive_result(o.state,r) {
                Receive::Save(w) => t.pending.update(n,Some(Proposed{write:w,request:Some(d)})), _ => t.pending },..t}
        },
        Step::Save{node,success} => {
            let p=t.pending[node as int].unwrap();let n=Completion{op:p.write.record.op,success};
            let out=complete_result(t.nodes[node as int].state,n);
            Trace{nodes:t.nodes.update(node as int,finisher(t.nodes[node as int],n)),pending:t.pending.update(node as int,None),
                sent:match out { Some(Action::Send(payload)) => t.sent.push(Sent{node:node as u64,write:p.write,notice:n,payload}), _ => t.sent },
                replies:match out { Some(Action::Reply(payload)) => t.replies.push(Returned{node:node as u64,request:p.request,payload}), _ => t.replies },..t}
        },
        Step::Deliver(i) => {
            let r=t.replies[i as int];let u=after_rpc(t,r.request.unwrap(),reply_result(r));
            Trace{delivered:t.delivered.push(i),..u}
        },
        Step::Error(d) => after_rpc(t,d,RpcResult::Error),
    }
}
pub open spec fn self_evidence(t: Trace, node:int, term:u64) -> bool {
    exists|i:int| 0 <= i < t.sent.len() && (#[trigger] t.sent[i]).node == node
        && t.sent[i].notice.success && t.sent[i].notice.op == t.sent[i].write.record.op
        && t.sent[i].payload.config == t.nodes[node].state.config
        && t.sent[i].payload.request.term == term && t.sent[i].payload.request.candidate == node
        && t.sent[i].write.record.state.term == term && t.sent[i].write.record.state.voted_for == Some(node as u64)
}
pub open spec fn remote_evidence(t: Trace, node:int, term:u64, voter:u64) -> bool {
    exists|j:int| 0 <= j < t.delivered.len() && {
        let i=#[trigger] t.delivered[j];
        &&& i < t.replies.len()
        &&& t.replies[i as int].request.is_some()
        &&& { let r=t.replies[i as int];let d=r.request.unwrap();
            &&& d.send < t.sent.len()
            &&& t.sent[d.send as int].node == node
            &&& t.sent[d.send as int].payload.request.candidate == node
            &&& t.sent[d.send as int].payload.request.term == term
            &&& d.peer == voter && r.node == voter && r.payload.node == voter
            &&& r.payload.candidate == node && r.payload.term == term && r.payload.granted
        }
    }
}
// Independent target. Neither this predicate nor any vote certificate is an input.
pub open spec fn sourced(t: Trace) -> bool {
    forall|a:int,v:u64| 0 <= a < t.nodes.len() && active(t.nodes[a].state)
        && #[trigger] t.nodes[a].votes.contains(v) ==> if v == a {
            self_evidence(t,a,t.nodes[a].state.state.term)
        } else { remote_evidence(t,a,t.nodes[a].state.state.term,v) }
}
// General finite nonempty configuration; identities are assigned by position.
pub fn new_network(initials: &Vec<Initial>) -> (n: Network)
    requires 0 < initials.len() <= u64::MAX,
        forall|i:int| 0 <= i < initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]),
    ensures boot(n.trace()), shape(n.trace()), n.trace().sent.len()==0, n.trace().replies.len()==0, n.trace().delivered.len()==0,
        n.trace().pending == Seq::new(initials.len() as nat,|i:int| None::<Proposed>),
        forall|i:int| 0 <= i < initials.len() ==> #[trigger] n.trace().nodes[i] == (Observation{
            state:fresh_view(Config{node:i as u64,members:initials.len() as u64},initials[i]),
            votes:Set::empty(),choices:Seq::empty(),base:None}),
        n.trace().nodes.len()==initials.len(),
{
    let mut nodes=Vec::new();let mut pending=Vec::new();let mut i:usize=0;
    while i < initials.len()
        invariant i <= initials.len(), 0 < initials.len() <= u64::MAX,
            forall|j:int| 0 <= j < initials.len() ==> valid_initial(Config{node:j as u64,members:initials.len() as u64},#[trigger] initials[j]),
            nodes.len()==i, pending@==Seq::new(i as nat,|k:int| None::<Proposed>),
            forall|k:int| 0 <= k < i ==> observe(#[trigger] nodes[k]) == (Observation{
                state:fresh_view(Config{node:k as u64,members:initials.len() as u64},initials[k]),
                votes:Set::empty(),choices:Seq::empty(),base:None}),
        decreases initials.len()-i,
    {
        nodes.push(Core::new(Config{node:i as u64,members:initials.len() as u64},initials[i]).unwrap());
        pending.push(None);i+=1;
    }
    Network{nodes,pending,sent:Vec::new(),replies:Vec::new(),delivered:Ghost(Seq::empty())}
}
// No authorization check on output contents: always record what actually returned.
// Save success is the disclosed faithful-I/O premise, not proof of physical I/O.
pub fn execute(n: &mut Network, s: Step)
    requires shape(old(n).trace()), enabled(old(n).trace(),s),
    ensures n.trace() == step_after(old(n).trace(),s), shape(n.trace()),
{
    let ghost before=n.trace();let ghost nodes_before=n.nodes@;
    let ghost expected=step_after(before,s);
    let at=match s {
        Step::Start(i) | Step::Save{node:i,success:_} => i,
        Step::Receive(d) => d.peer as usize,
        Step::Error(d) => n.sent[d.send].node as usize,
        Step::Deliver(i) => n.sent[n.replies[i].request.unwrap().send].node as usize,
    };
    assert(at < n.nodes.len());
    let mut core=n.nodes.remove(at);
    match s {
        Step::Start(_) => {
            let out=core.start_election();
            if let Campaign::Save(w)=out { n.pending[at]=Some(Proposed{write:w,request:None}); }
        },
        Step::Receive(d) => {
            let r=n.sent[d.send].payload.request;let out=core.receive(r);
            if let Receive::Save(w)=out { n.pending[at]=Some(Proposed{write:w,request:Some(d)}); }
        },
        Step::Save{node,success} => {
            let p=n.pending[node].unwrap();let notice=Completion{op:p.write.record.op,success};
            let out=core.complete(notice);n.pending[node]=None;
            match out {
                Some(Action::Send(payload)) => n.sent.push(Sent{node:node as u64,write:p.write,notice,payload}),
                Some(Action::Reply(payload)) => n.replies.push(Returned{node:node as u64,request:p.request,payload}),
                _ => {},
            }
        },
        Step::Deliver(i) => {
            let r=n.replies[i];let d=r.request.unwrap();let sent=n.sent[d.send];
            let out=core.collect(RpcContext{sent:sent.payload,peer:d.peer},
                RpcResult::Reply{term:r.payload.term,granted:r.payload.granted});
            if let Collect::Save(w)=out { n.pending[at]=Some(Proposed{write:w,request:None}); }
            proof { n.delivered@=n.delivered@.push(i); }
        },
        Step::Error(d) => {
            let sent=n.sent[d.send];let out=core.collect(RpcContext{sent:sent.payload,peer:d.peer},RpcResult::Error);
            if let Collect::Save(w)=out { n.pending[at]=Some(Proposed{write:w,request:None}); }
        },
    }
    assert(observe(core)==expected.nodes[at as int]);
    assert(n.pending@==expected.pending);
    assert(n.sent@==expected.sent && n.replies@==expected.replies && n.delivered@==expected.delivered);
    n.nodes.insert(at,core);
    assert forall|j:int| 0<=j<n.nodes.len() implies #[trigger] n.trace().nodes[j]==expected.nodes[j] by {
        if j != at { assert(n.nodes@[j]==nodes_before[j]); }
    }
    assert(n.trace().nodes =~= expected.nodes);
    assert forall|j:int| 0<=j<n.nodes.len() implies (#[trigger] n.trace().nodes[j]).state.config==Config{node:j as u64,members:n.nodes.len() as u64} by {
        if j != at { assert(n.nodes@[j]==nodes_before[j]); }
    }
    assert forall|j:int| 0<=j<n.sent.len() implies (#[trigger] n.sent@[j]).node<n.nodes.len() by {
        if j<before.sent.len() { assert(n.sent@[j]==before.sent[j]); }
    }
}

pub fn is_enabled(n:&Network,s:Step) -> (ok:bool)
    requires shape(n.trace()),
    ensures ok==enabled(n.trace(),s),
{
    match s {
        Step::Start(i) => i<n.nodes.len(),
        Step::Receive(d) | Step::Error(d) => d.send<n.sent.len() && d.peer<n.nodes.len() as u64 && d.peer!=n.sent[d.send].node,
        Step::Save{node,success:_} => node<n.nodes.len() && n.pending[node].is_some(),
        Step::Deliver(i) => {
            if !has_reply(&n.replies,i) { false }
            else if let Some(d)=n.replies[i].request {
                d.send<n.sent.len() && d.peer<n.nodes.len() as u64 && d.peer!=n.sent[d.send].node
            } else { false }
        },
    }
}

pub fn query(n:&Network,node:usize,member:u64) -> (out:(u64,bool))
    requires node<n.nodes.len(),
    ensures out.0 as nat==n.trace().nodes[node as int].votes.len(),
        out.1==n.trace().nodes[node as int].votes.contains(member),
        out.0<=n.trace().nodes[node as int].state.config.members,
{
    let c=&n.nodes[node];proof { c.domain(); }
    (c.vote_count(),c.has_vote(member))
}

// Structural availability only, never an assertion that a protocol output is right.
pub fn has_reply(replies:&Vec<Returned>,i:usize) -> (ok:bool)
    ensures ok==(i<replies.len()),
{ i<replies.len() }

}
