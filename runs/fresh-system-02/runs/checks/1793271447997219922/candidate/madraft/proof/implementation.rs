//! Partial cluster adapter. Async completion and crash coverage remain OPEN.
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "../src/raft/log.rs"]
pub mod log_core;
#[path = "../src/raft/quorum.rs"]
pub mod quorum;
#[path = "../src/raft/vote_history.rs"]
pub mod vote_history;
#[path = "segments.rs"]
mod segments;
use crate::{support::*, targets};
use vstd::prelude::*;
verus! {
pub type Message = segments::Message;
pub type Output = segments::Output;
pub type State = Map<targets::Key,segments::State>;
pub struct RuntimeNode { pub group:u64,pub node:usize,pub members:usize,pub votes:Seq<bool>,pub history:Map<u64,u64>,pub election:election::Election,
    pub log:Seq<log_core::Entry>,pub commit:u64,pub applied:u64 }
pub type RuntimeState = Map<targets::Key,RuntimeNode>;
pub open spec fn core_state(c:log_core::Core) -> RuntimeNode {
    RuntimeNode {group:c.group,node:c.node,members:c.members,votes:c.votes@,history:c.vote.history@,election:c.vote.current,log:c.log@,commit:c.commit,applied:c.applied}
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
    segments::State {group:c.group as nat,node:c.node as nat,members:c.members as nat,votes:c.votes,history:c.history,election:c.election,log:c.log,commit:c.commit,applied:c.applied}
}
pub open spec fn identified(k: targets::Key,s: segments::State) -> bool { k.group == s.group && k.node == s.node }
pub open spec fn membership(s:State) -> bool {
    (forall|a:targets::Key,b:targets::Key| s.contains_key(a) && s.contains_key(b) && a.group==b.group
        ==> s[a].members==s[b].members)
    && (forall|a:targets::Key,j:nat| s.contains_key(a) && j<s[a].members ==>
        s.contains_key(targets::Key {group:a.group,node:j}))
}
pub open spec fn initial(s: State) -> bool {
    membership(s) && s.dom().finite() && forall|k: targets::Key| s.contains_key(k) ==> identified(k,s[k]) && segments::initial(s[k])
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
pub open spec fn vote_backed(s:State,m:Envelope<Message>) -> bool {
    match m.body {
        Message::Response {reply,request_term,candidate} => !reply.granted || {
            let k=targets::Key {group:m.group,node:m.sender};
            s.contains_key(k) && s[k].history.contains_key(request_term) && s[k].history[request_term]==candidate
        },
        _ => true,
    }
}
pub open spec fn sends_backed(s:State,sent:Seq<Envelope<Message>>) -> bool {
    forall|j:int|0<=j<sent.len() ==> vote_backed(s,#[trigger] sent[j])
}
pub proof fn preserve_sends(s:State,i:Input,t:State,o:Output,a:Seq<Envelope<Message>>,b:Seq<Envelope<Message>>)
    requires step(s,i,t,o),sends_backed(s,a),exchange(a,receive(i),o,b),
        identified(i.who,s[i.who]),segments::valid_history(s[i.who]),
    ensures sends_backed(t,b),
{
    segments::preserve_history(s[i.who],i.action,t[i.who],o);
    assert forall|j:int|0<=j<b.len() implies vote_backed(t,#[trigger] b[j]) by {
        if j<a.len() {
            assert(b[j]==a[j]);
            match a[j].body {
                Message::Response {reply,request_term,candidate} => {
                    let k=targets::Key {group:a[j].group,node:a[j].sender};
                    if reply.granted {
                        assert(vote_backed(s,a[j]));
                        if k==i.who {assert(t[k].history[request_term]==s[k].history[request_term]);}
                        else {assert(t[k]==s[k]);}
                    }
                }, _=>{},
            }
        } else {
            match i.action {_=>{}}
            assert(o.len()<=1);
            assert(j==a.len());
            assert(b[j]==o[0]);
        }
    }
}
pub open spec fn active(s:segments::State) -> bool {s.election.role is Candidate || s.election.role is Leader}
pub open spec fn back_vote(s:State,g:nat,voter:nat,term:u64,candidate:nat) -> bool {
    let k=targets::Key {group:g,node:voter};
    s.contains_key(k) && s[k].history.contains_key(term) && s[k].history[term]==candidate
}
pub open spec fn certificates(s:State) -> bool {
    forall|k:targets::Key,j:int| s.contains_key(k) && active(s[k]) && 0<=j<s[k].votes.len() && #[trigger] s[k].votes[j]
        ==> back_vote(s,k.group,j as nat,s[k].election.term,k.node)
}
pub open spec fn self_votes(s:State) -> bool {
    forall|k:targets::Key| s.contains_key(k) && active(s[k]) ==>
        s[k].election.voted_for==Some(k.node as u64) && s[k].votes[k.node as int]
}
pub open spec fn histories_extend(s:State,t:State) -> bool {
    forall|k:targets::Key| s.contains_key(k) ==> t.contains_key(k) && vote_history::extends(s[k].history,t[k].history)
}
pub proof fn back_vote_extends(s:State,t:State,g:nat,v:nat,term:u64,c:nat)
    requires histories_extend(s,t),back_vote(s,g,v,term,c),
    ensures back_vote(t,g,v,term,c),
{ let k=targets::Key {group:g,node:v}; assert(vote_history::extends(s[k].history,t[k].history)); }
pub proof fn preserve_certificates(s:State,i:Input,t:State,o:Output,a:Seq<Envelope<Message>>,b:Seq<Envelope<Message>>)
    requires step(s,i,t,o),exchange(a,receive(i),o,b),sends_backed(s,a),certificates(s),self_votes(s),histories_extend(s,t),
        forall|k:targets::Key|s.contains_key(k) ==> identified(k,s[k]) && segments::structure(s[k]) && segments::valid_history(s[k]),
    ensures certificates(t),self_votes(t),
{
    assert forall|k:targets::Key|t.contains_key(k) && active(t[k]) implies
        t[k].election.voted_for==Some(k.node as u64) && t[k].votes[k.node as int] by {
        if k!=i.who {assert(t[k]==s[k]);} else {match i.action {_=>{}}}
    }
    assert forall|k:targets::Key,j:int| t.contains_key(k) && active(t[k]) && 0<=j<t[k].votes.len() && #[trigger] t[k].votes[j]
        implies back_vote(t,k.group,j as nat,t[k].election.term,k.node) by {
        if k!=i.who {
            assert(t[k]==s[k]);
            assert(back_vote(s,k.group,j as nat,s[k].election.term,k.node));
            back_vote_extends(s,t,k.group,j as nat,s[k].election.term,k.node);
        } else {
            match i.action {
                segments::Input::Timeout {node} => {
                    assert(j==k.node);
                    assert(t[k].history.contains_key(t[k].election.term));
                },
                segments::Input::VoteReplyDelivery {delivery:d,started,..} => {
                    match d.message.body {
                        Message::Response {reply,request_term,candidate} => {
                            if s[k].votes[j] {
                                assert(active(s[k]) && t[k].election.term==s[k].election.term);
                                back_vote_extends(s,t,k.group,j as nat,s[k].election.term,k.node);
                            } else {
                                assert(d.message.sender==j);
                                assert(d.message==a[d.send as int]);
                                assert(vote_backed(s,d.message));
                                back_vote_extends(s,t,k.group,j as nat,request_term,candidate as nat);
                            }
                        },_=>{},
                    }
                },
                _ => {
                    assert(active(s[k]) && t[k].election.term==s[k].election.term && s[k].votes[j]);
                    back_vote_extends(s,t,k.group,j as nat,s[k].election.term,k.node);
                },
            }
        }
    }
}
pub open spec fn majorities(s:State) -> bool {
    forall|k:targets::Key|s.contains_key(k) && s[k].election.role is Leader ==>
        quorum::tally(s[k].votes,s[k].votes.len())>s[k].members/2
}
pub proof fn preserve_majorities(s:State,i:Input,t:State,o:Output)
    requires step(s,i,t,o),majorities(s),self_votes(s),
        forall|k:targets::Key|s.contains_key(k) ==> identified(k,s[k]) && segments::structure(s[k]),
    ensures majorities(t),
{
    assert forall|k:targets::Key|t.contains_key(k) && t[k].election.role is Leader implies
        quorum::tally(t[k].votes,t[k].votes.len())>t[k].members/2 by {
        if k!=i.who {assert(t[k]==s[k]);} else {match i.action {_=>{}}}
    }
}
pub open spec fn model_invariant(s: State,sent: Seq<Envelope<Message>>) -> bool {
    majorities(s) && certificates(s) && self_votes(s) && membership(s) && sends_backed(s,sent) && s.dom().finite() && forall|k:targets::Key| s.contains_key(k) ==>
        identified(k,s[k]) && segments::structure(s[k]) && segments::valid_history(s[k])
}
pub open spec fn runtime_invariant(s:RuntimeState,sent:Seq<Envelope<Message>>) -> bool {model_invariant(view(s),sent)}
pub proof fn prove_model_initial(s:State)
    requires initial(s),ensures model_invariant(s,Seq::empty()),
{}
pub proof fn prove_runtime_initial(s: RuntimeState)
    requires runtime_initial(s), ensures runtime_invariant(s,Seq::empty()), initial(view(s)),
{}
pub proof fn prove_model_step(s:State,i:Input,t:State,o:Output,a:Seq<Envelope<Message>>,b:Seq<Envelope<Message>>)
    requires model_invariant(s,a),step(s,i,t,o),exchange(a,receive(i),emit(o),b),
    ensures model_invariant(t,b),histories_extend(s,t),t.dom()==s.dom(),
        forall|k:targets::Key|s.contains_key(k) ==> t[k].members==s[k].members,
{
    assert(s.contains_key(i.who));
    assert(segments::structure(s[i.who]));
    segments::preserve_structure(s[i.who],i.action,t[i.who],o);
    segments::preserve_history(s[i.who],i.action,t[i.who],o);
    assert(histories_extend(s,t)) by {
        assert forall|k:targets::Key| s.contains_key(k) implies t.contains_key(k)
            && vote_history::extends(s[k].history,t[k].history) by {if k!=i.who {assert(t[k]==s[k]);}}
    }
    preserve_certificates(s,i,t,o,a,b);
    preserve_sends(s,i,t,o,a,b);
    assert(t.dom() =~= s.dom());
    assert(membership(t)) by {
        assert forall|k:targets::Key|s.contains_key(k) implies t[k].members==s[k].members by {if k!=i.who {assert(t[k]==s[k]);}}
    }
    assert forall|k:targets::Key| t.contains_key(k) implies
        identified(k,t[k]) && segments::structure(t[k]) && segments::valid_history(t[k]) by {
        if k != i.who { assert(t[k]==s[k]); }
    }

    preserve_majorities(s,i,t,o);
}
pub proof fn prove_runtime_step(s:RuntimeState,i:Input,t:RuntimeState,o:Output,a:Seq<Envelope<Message>>,b:Seq<Envelope<Message>>)
    requires runtime_invariant(s,a),runtime_step(s,i,t,o),exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b),step(view(s),i,view(t),o),
{
    assert(view(t) =~= view(s).insert(i.who,node_view(t[i.who])));
    prove_model_step(view(s),i,view(t),o,a,b);
}
pub fn actual_initial(group: u64,node: usize,members:usize) -> (c: log_core::Core)
    requires node < members,
    ensures segments::initial(node_view(core_state(c))),identified(targets::Key {group:group as nat,node:node as nat},node_view(core_state(c))),
{ log_core::new_core(group,node,members) }
pub fn actual_proposal(c: &mut log_core::Core,entry: log_core::Entry) -> (ok: bool)
    requires old(c).vote.current.role == election::Role::Leader,
    ensures ok ==> segments::step(node_view(core_state(*old(c))),segments::Input::Propose {entry},node_view(core_state(*c)),Seq::empty()),
        !ok ==> c.log@ == old(c).log@,
{ log_core::propose(&mut c.log,entry) }
pub fn actual_insert(c: &mut log_core::Core,at: usize,entry: log_core::Entry) -> (ok: bool)
    requires !(old(c).vote.current.role == election::Role::Leader),
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
    let c=RuntimeNode {group:1,node:0,members:1,votes:seq![false],history:Map::empty(),election:election::fresh(),log:Seq::empty(),commit:0,applied:0};
    let candidate=RuntimeNode {votes:seq![true],history:c.history.insert(1,0),election:election::timeout_result(c.election,0),..c};
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
    assert(membership(view(s))) by {
        assert forall|a:targets::Key,j:nat| view(s).contains_key(a) && j<view(s)[a].members
            implies view(s).contains_key(targets::Key {group:a.group,node:j}) by {assert(j==0);}
    }
    assert(runtime_initial(s));
    assert forall|k:targets::Key| view(s).contains_key(k) implies identified(k,view(s)[k]) && segments::initial(view(s)[k]) by {}
    assert(segments::next_votes(node_view(c),segments::Input::Timeout {node:0}) =~= seq![true]);
    assert forall|j:int| 0 <= j < h.inputs.len() implies runtime_step(h.states[j],#[trigger] h.inputs[j],h.states[j+1],h.outputs[j])
        && exchange(h.sent[j],receive(h.inputs[j]),emit(h.outputs[j]),h.sent[j+1]) by {
        assert(j==0 || j==1);
    }
    assert(runtime_execution(h));
    prove_runtime_initial(s);
    prove_runtime_step(s,h.inputs[0],t,empty,empty,empty);
    prove_runtime_step(t,h.inputs[1],u,empty,empty,empty);
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

pub proof fn invariant_at(h:crate::Execution,k:int)
    requires crate::execution(h),0<=k<h.states.len(),
    ensures model_invariant(h.states[k],h.sent[k]),
{
    assert forall|s:State|initial(s) implies #[trigger] model_invariant(s,Seq::empty()) by {prove_model_initial(s);}
    assert forall|s:State,i:Input,t:State,o:Output,a:Seq<Envelope<Message>>,b:Seq<Envelope<Message>>|
        #[trigger] model_invariant(s,a) && #[trigger] step(s,i,t,o) && #[trigger] exchange(a,receive(i),emit(o),b)
        implies model_invariant(t,b) by {prove_model_step(s,i,t,o,a,b);}
    crate::support::invariant_at(h,|s|initial(s),|s,i,t,o|step(s,i,t,o),|i|receive(i),|o|emit(o),|s,a|model_invariant(s,a),k);
}
pub proof fn histories_between(h:crate::Execution,a:int,b:int)
    requires crate::execution(h),0<=a<=b<h.states.len(),
    ensures histories_extend(h.states[a],h.states[b]),h.states[a].dom()==h.states[b].dom(),
        forall|k:targets::Key|h.states[a].contains_key(k) ==> h.states[a][k].members==h.states[b][k].members,
    decreases b-a,
{
    if a<b {
        histories_between(h,a,b-1);invariant_at(h,b-1);
        prove_model_step(h.states[b-1],h.inputs[b-1],h.states[b],h.outputs[b-1],h.sent[b-1],h.sent[b]);
        assert forall|k:targets::Key|h.states[a].contains_key(k) implies h.states[b].contains_key(k)
            && vote_history::extends(h.states[a][k].history,h.states[b][k].history) by {
            assert(vote_history::extends(h.states[a][k].history,h.states[b-1][k].history));
            assert(vote_history::extends(h.states[b-1][k].history,h.states[b][k].history));
        }
    }
}
pub open spec fn vote_at(s:State,g:nat,v:nat,term:u64) -> Option<nat> {
    let k=targets::Key {group:g,node:v};
    if s.contains_key(k) && s[k].history.contains_key(term) {Some(s[k].history[term] as nat)} else {None}
}
pub proof fn prove_election_safety(h:crate::Execution)
    requires crate::execution(h),ensures targets::election_safety(observe(h)),
{
    let obs=observe(h);
    assert forall|i:int,j:int| #![trigger obs.events[i],obs.events[j]]
        0<=i<obs.events.len() && 0<=j<obs.events.len() implies match (obs.events[i],obs.events[j]) {
        (targets::Event::Elect {who:a,term:ta},targets::Event::Elect {who:b,term:tb}) =>
            a.group==b.group && ta==tb ==> a.node==b.node,
        _=>true,
    } by {
        match (obs.events[i],obs.events[j]) {
            (targets::Event::Elect {who:a,term:ta},targets::Event::Elect {who:b,term:tb}) => {
                if a.group==b.group && ta==tb {
                    let x=i+1;let y=j+1;let z=if x>y {x} else {y};
                    invariant_at(h,x);invariant_at(h,y);invariant_at(h,z);
                    histories_between(h,x,z);histories_between(h,y,z);
                    let va=h.states[x][a].votes;let vb=h.states[y][b].votes;
                    let term=h.states[x][a].election.term;
                    assert(h.states[x].contains_key(a) && h.states[y].contains_key(b));
                    assert(va.len()==vb.len());
                    assert forall|v:int|0<=v<va.len() && #[trigger] va[v] implies vote_at(h.states[z],a.group,v as nat,term)==Some(a.node) by {
                        assert(back_vote(h.states[x],a.group,v as nat,term,a.node));
                        back_vote_extends(h.states[x],h.states[z],a.group,v as nat,term,a.node);
                    }
                    assert forall|v:int|0<=v<vb.len() && #[trigger] vb[v] implies vote_at(h.states[z],a.group,v as nat,term)==Some(b.node) by {
                        assert(back_vote(h.states[y],b.group,v as nat,term,b.node));
                        back_vote_extends(h.states[y],h.states[z],b.group,v as nat,term,b.node);
                    }
                    quorum::certificate_unique(va,vb,a.node,b.node,|v:nat|vote_at(h.states[z],a.group,v,term));
                }
            },_=>{},
        }
    }
}

}
