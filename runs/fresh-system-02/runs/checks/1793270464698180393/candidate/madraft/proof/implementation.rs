//! Partial cluster adapter. Async completion and crash coverage remain OPEN.
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "../src/raft/log.rs"]
pub mod log_core;
#[path = "../src/raft/quorum.rs"]
pub mod quorum;
#[path = "segments.rs"]
mod segments;
use crate::{support::*, targets};
use vstd::prelude::*;
verus! {
pub type Message = segments::Message;
pub type Output = segments::Output;
pub type State = Map<targets::Key,segments::State>;
pub struct RuntimeNode { pub group:u64,pub node:usize,pub members:usize,pub votes:Seq<bool>,pub election:election::Election,
    pub log:Seq<log_core::Entry>,pub commit:u64,pub applied:u64 }
pub type RuntimeState = Map<targets::Key,RuntimeNode>;
pub open spec fn core_state(c:log_core::Core) -> RuntimeNode {
    RuntimeNode {group:c.group,node:c.node,members:c.members,votes:c.votes@,election:c.election,log:c.log@,commit:c.commit,applied:c.applied}
}
pub type RuntimeHistory = History<RuntimeState,Input,Output,Message>;
pub open spec fn runtime_execution(h:RuntimeHistory) -> bool {
    valid(h,|s|runtime_initial(s),|s,i,t,o|runtime_step(s,i,t,o),|i|receive(i),|o|emit(o))
}
pub open spec fn abstract_execution(h:RuntimeHistory) -> crate::Execution { project(h,|s|view(s)) }
pub struct Input { pub who: targets::Key, pub action: segments::Input }
pub open spec fn receive(i: Input) -> Option<Delivery<Message>> { segments::receive(i.action) }
pub open spec fn emit(o: Output) -> Seq<Envelope<Message>> { o }
pub open spec fn node_view(c: RuntimeNode) -> segments::State {
    segments::State {group:c.group as nat,node:c.node as nat,members:c.members as nat,votes:c.votes,election:c.election,log:c.log,commit:c.commit,applied:c.applied}
}
pub open spec fn identified(k: targets::Key,s: segments::State) -> bool { k.group == s.group && k.node == s.node }
pub open spec fn initial(s: State) -> bool {
    s.dom().finite() && forall|k: targets::Key| s.contains_key(k) ==> identified(k,s[k]) && segments::initial(s[k])
}
pub open spec fn step(s: State,i: Input,t: State,o: Output) -> bool {
    s.contains_key(i.who) && t.contains_key(i.who)
    && t == s.insert(i.who,t[i.who])
    && segments::step(s[i.who],i.action,t[i.who],o)
}
pub open spec fn view(s: RuntimeState) -> State {
    Map::new(|k|s.contains_key(k),|k|node_view(s[k]))
}
pub open spec fn runtime_initial(s: RuntimeState) -> bool { initial(view(s)) }
pub open spec fn runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output) -> bool {
    s.contains_key(i.who) && t.contains_key(i.who) && t == s.insert(i.who,t[i.who])
    && segments::step(node_view(s[i.who]),i.action,node_view(t[i.who]),o)
}
pub open spec fn runtime_invariant(s: RuntimeState,sent: Seq<Envelope<Message>>) -> bool { true }
pub proof fn prove_runtime_initial(s: RuntimeState)
    requires runtime_initial(s), ensures runtime_invariant(s,Seq::empty()), initial(view(s)),
{}
pub proof fn prove_runtime_step(s: RuntimeState,i: Input,t: RuntimeState,o: Output,a: Seq<Envelope<Message>>,b: Seq<Envelope<Message>>)
    requires runtime_invariant(s,a),runtime_step(s,i,t,o),exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b),step(view(s),i,view(t),o),
{
    assert(view(t) =~= view(s).insert(i.who,node_view(t[i.who])));
}
pub fn actual_initial(group: u64,node: usize,members:usize) -> (c: log_core::Core)
    requires node < members,
    ensures segments::initial(node_view(core_state(c))),identified(targets::Key {group:group as nat,node:node as nat},node_view(core_state(c))),
{ log_core::new_core(group,node,members) }
pub fn actual_proposal(c: &mut log_core::Core,entry: log_core::Entry) -> (ok: bool)
    requires old(c).election.role == election::Role::Leader,
    ensures ok ==> segments::step(node_view(core_state(*old(c))),segments::Input::Propose {entry},node_view(core_state(*c)),Seq::empty()),
        !ok ==> c.log@ == old(c).log@,
{ log_core::propose(&mut c.log,entry) }
pub fn actual_insert(c: &mut log_core::Core,at: usize,entry: log_core::Entry) -> (ok: bool)
    requires !(old(c).election.role == election::Role::Leader),
    ensures segments::step(node_view(core_state(*old(c))),segments::Input::Insert {at:at as nat,entry},node_view(core_state(*c)),Seq::empty()),
{ log_core::insert(&mut c.log,at,entry) }
pub open spec fn observed_log(log: Seq<log_core::Entry>) -> Seq<targets::Entry> {
    Seq::new(log.len(),|j: int|targets::Entry {term:log[j].term as nat,command:log[j].command@})
}
pub open spec fn observed_state(s: State) -> Map<targets::Key,targets::View> {
    Map::new(|k|s.contains_key(k),|k| targets::View {term:s[k].election.term as nat,
        leader:s[k].election.role is Leader,log:observed_log(s[k].log),snapshot:None})
}
pub open spec fn event(s: State,i: Input,t: State) -> targets::Event {
    if !(s[i.who].election.role is Leader) && t[i.who].election.role is Leader {
        targets::Event::Elect {who:i.who,term:t[i.who].election.term as nat}
    } else if t[i.who].commit > s[i.who].commit {
        targets::Event::Commit {who:i.who,term:t[i.who].election.term as nat,through:t[i.who].commit as nat}
    } else if t[i.who].applied > s[i.who].applied {
        targets::Event::Apply {who:i.who,index:t[i.who].applied as nat,
            entry:observed_log(t[i.who].log)[t[i.who].applied as int-1]}
    } else {targets::Event::Silent}
}
pub open spec fn observe(h: crate::Execution) -> targets::History {
    targets::History {states:Seq::new(h.states.len(),|j: int|observed_state(h.states[j])),
        events:Seq::new(h.inputs.len(),|j: int|event(h.states[j],h.inputs[j],h.states[j+1]))}
}
pub proof fn log_prefix_map(a: Seq<log_core::Entry>,b: Seq<log_core::Entry>)
    requires a.len() <= b.len(),b.take(a.len() as int) == a,
    ensures targets::prefix(observed_log(a),observed_log(b)),
{
    assert(observed_log(a) =~= observed_log(b).take(a.len() as int));
}
pub proof fn prove_leader_append_only(h: crate::Execution)
    requires crate::execution(h), ensures targets::leader_append_only(observe(h)),
{
    let obs = observe(h);
    assert forall|j: int,k: targets::Key| #![trigger obs.states[j][k]]
        0 <= j < obs.events.len() && obs.states[j].contains_key(k) && obs.states[j+1].contains_key(k)
        && obs.states[j][k].leader && obs.states[j+1][k].leader && obs.states[j][k].term == obs.states[j+1][k].term
        implies targets::prefix(obs.states[j][k].log,obs.states[j+1][k].log) by {
        let i = h.inputs[j];
        assert(step(h.states[j],i,h.states[j+1],h.outputs[j]));
        if k == i.who {
            segments::local_leader_append_only(h.states[j][k],i.action,h.states[j+1][k],h.outputs[j]);
        } else {
            assert(h.states[j+1][k] == h.states[j][k]);
            assert(h.states[j+1][k].log.take(h.states[j][k].log.len() as int) =~= h.states[j][k].log);
        }
        log_prefix_map(h.states[j][k].log,h.states[j+1][k].log);
    }
}
pub open spec fn applications() -> Map<nat,targets::Application> { Map::empty() }
pub proof fn prove_observation_and_snapshots(h:crate::Execution)
    requires crate::execution(h),
    ensures targets::shape(observe(h)),targets::snapshot_fidelity(observe(h),applications()),
{
    let obs = observe(h);
    assert forall|j:int| 0 <= j < obs.events.len() implies match #[trigger] obs.events[j] {
        targets::Event::Elect {who,term} => obs.states[j+1].contains_key(who)
            && obs.states[j+1][who].leader && obs.states[j+1][who].term == term,
        targets::Event::Commit {who,term,through} => obs.states[j+1].contains_key(who)
            && obs.states[j+1][who].term == term && 1 <= through <= obs.states[j+1][who].log.len(),
        targets::Event::Apply {index,..} => index >= 1,
        _ => true,
    } by {
        let i=h.inputs[j];
        assert(step(h.states[j],i,h.states[j+1],h.outputs[j]));
        match i.action { _ => {} }
    }
    assert forall|j:int,k:targets::Key| #![trigger obs.states[j][k]]
        0 <= j < obs.states.len() && obs.states[j].contains_key(k)
        implies obs.states[j][k].snapshot == None::<targets::Snapshot> by {}
    assert forall|j:int| 0 <= j < obs.events.len() implies match #[trigger] obs.events[j] {
        targets::Event::ApplySnapshot {..} | targets::Event::CreateSnapshot {..}
        | targets::Event::PersistSnapshot {..} | targets::Event::RestoreSnapshot {..} => false,
        _ => true,
    } by {}
}
pub proof fn execution_witness() -> (h:RuntimeHistory)
    ensures runtime_execution(h), targets::shape(observe(abstract_execution(h))),
        exists|j:int| 0 <= j < observe(abstract_execution(h)).events.len()
            && !(observe(abstract_execution(h)).events[j] is Silent),
{
    let key=targets::Key {group:1,node:0};
    let c=RuntimeNode {group:1,node:0,members:1,votes:seq![false],election:election::fresh(),log:Seq::empty(),commit:0,applied:0};
    let candidate=RuntimeNode {votes:seq![true],election:election::timeout_result(c.election,0),..c};
    let leader=RuntimeNode {election:election::Election {role:election::Role::Leader,..candidate.election},..candidate};
    let s=Map::empty().insert(key,c);
    let t=s.insert(key,candidate);
    let u=t.insert(key,leader);
    let empty=Seq::<Envelope<Message>>::empty();
    let h=History {states:seq![s,t,u],inputs:seq![Input {who:key,action:segments::Input::Timeout {node:0}},
        Input {who:key,action:segments::Input::SelfElect}],outputs:seq![empty,empty],sent:seq![empty,empty,empty]};
    assert(s.dom() =~= set![key]);
    assert(s.dom().finite());
    assert(view(s).dom() =~= s.dom());
    assert(runtime_initial(s));
    assert forall|k:targets::Key| view(s).contains_key(k) implies identified(k,view(s)[k]) && segments::initial(view(s)[k]) by {}
    assert forall|j:int| 0 <= j < h.inputs.len() implies runtime_step(h.states[j],#[trigger] h.inputs[j],h.states[j+1],h.outputs[j])
        && exchange(h.sent[j],receive(h.inputs[j]),emit(h.outputs[j]),h.sent[j+1]) by {
        assert(j==0 || j==1);
    }
    assert(runtime_execution(h));
    assert(crate::execution(abstract_execution(h))) by {
        assert forall|j:int| 0 <= j < h.inputs.len() implies step(view(h.states[j]),#[trigger] h.inputs[j],view(h.states[j+1]),h.outputs[j]) by {
            assert(j==0 || j==1);
            prove_runtime_step(h.states[j],h.inputs[j],h.states[j+1],h.outputs[j],empty,empty);
        }
    }
    prove_observation_and_snapshots(abstract_execution(h));
    assert(!(observe(abstract_execution(h)).events[1] is Silent));
    h
}

}
