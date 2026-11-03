//! Inductive evidence for the fixed no-restart executor's actual Core calls.
use crate::{vote_api::*,vote_network::*,vote_core::history_ok,election_observation};
use vstd::prelude::*;
verus! {
broadcast use {vstd::seq_lib::group_seq_properties,vstd::set::group_set_axioms};
pub open spec fn local(o:Observation) -> bool {
    &&& valid_state(o.state.config,o.state.state)
    &&& history_ok(o.state,o.choices,o.base)
    &&& voters_domain(o.state,o.votes)
    &&& o.state.pending.is_some() ==> {
        let w=o.state.pending.unwrap();
        &&& valid_record(o.state.config,w.record) && w.record.state==o.state.state && w.record.op<o.state.next
        &&& match w.action {
            Action::Send(s) => s.config==o.state.config && valid_request(s.config,s.request)
                && s.request.candidate==s.config.node && s.request.term==o.state.state.term
                && s.request.log==o.state.state.log && o.state.state.voted_for==Some(s.config.node)
                && o.state.role==Role::Candidate && o.state.round==Some(o.state.state.term) && !o.state.halted
                && o.choices.contains(Choice{node:s.config.node,term:s.request.term,candidate:s.config.node}),
            Action::Reply(r) => r.node==o.state.config.node && r.term==o.state.state.term
                && (r.granted ==> o.state.state.voted_for==Some(r.candidate)),
            _=>true,
        }
    }
}
pub open spec fn pending_ok(t:Trace,n:int) -> bool {
    let p=t.pending[n];let o=t.nodes[n];
    &&& o.state.pending==match p {Some(p)=>Some(p.write),None=>None}
    &&& p.is_some() ==> {
        let p=p.unwrap();
        match p.write.action {
            Action::Reply(r)=> p.request.is_some() && routed(t,p.request.unwrap())
                && p.request.unwrap().peer==n
                && p.write.request==Some(t.sent[p.request.unwrap().send as int].payload.request)
                && r.candidate==t.sent[p.request.unwrap().send as int].payload.request.candidate
                && (r.granted ==> r.term==t.sent[p.request.unwrap().send as int].payload.request.term),
            _=>p.request.is_none(),
        }
    }
}
pub open spec fn sent_ok(t:Trace,i:int) -> bool {
    let s=t.sent[i];
    &&& s.node<t.nodes.len() && s.payload.config==t.nodes[s.node as int].state.config
    &&& valid_request(s.payload.config,s.payload.request) && s.payload.request.candidate==s.node
    &&& s.notice.success && s.notice.op==s.write.record.op
    &&& s.write.action==Action::Send(s.payload)
    &&& s.write.record.state.term==s.payload.request.term && s.write.record.state.voted_for==Some(s.node)
    &&& choice_for(t,s.node,s.node,s.payload.request.term)
}
pub open spec fn returned_ok(t:Trace,i:int) -> bool {
    let r=t.replies[i];
    &&& r.request.is_some() && routed(t,r.request.unwrap())
    &&& r.node==r.request.unwrap().peer && r.payload.node==r.node
    &&& r.payload.candidate==t.sent[r.request.unwrap().send as int].payload.request.candidate
    &&& r.payload.granted ==> r.payload.term==t.sent[r.request.unwrap().send as int].payload.request.term
        && choice_for(t,r.node,r.payload.candidate,r.payload.term)
}
pub open spec fn all_sourced(t:Trace) -> bool {
    forall|a:int,v:u64| 0<=a<t.nodes.len() && #[trigger] t.nodes[a].votes.contains(v)
        ==> if v==a {self_evidence(t,a,t.nodes[a].state.state.term)} else {remote_evidence(t,a,t.nodes[a].state.state.term,v)}
}
pub open spec fn inv(t:Trace) -> bool {
    &&& forall|n:int| 0<=n<t.nodes.len() ==> local(#[trigger] t.nodes[n]) && pending_ok(t,n)
    &&& forall|i:int| 0<=i<t.sent.len() ==> sent_ok(t,i)
    &&& forall|i:int| 0<=i<t.replies.len() ==> returned_ok(t,i)
    &&& all_sourced(t) && choice_sourced(t) && elections_justified(t)
}
pub proof fn initialize(t:Trace)
    requires boot(t),ensures inv(t),
{
    assert forall|n:int| 0<=n<t.nodes.len() implies local(#[trigger] t.nodes[n]) && pending_ok(t,n) by {
        assert(t.pending[n].is_none());
        assert(t.nodes[n].choices==Seq::<Choice>::empty());
        assert(t.nodes[n].votes==Set::<u64>::empty());
        assert(t.nodes[n].base.is_none());
    }
}
pub open spec fn extends(a:Trace,b:Trace) -> bool {
    &&& a.nodes.len()==b.nodes.len()
    &&& a.sent.is_prefix_of(b.sent) && a.replies.is_prefix_of(b.replies) && a.delivered.is_prefix_of(b.delivered)
    &&& forall|n:int| 0<=n<a.nodes.len() ==> (#[trigger] a.nodes[n]).state.config==b.nodes[n].state.config
        && a.nodes[n].choices.is_prefix_of(b.nodes[n].choices)
}
#[verifier::spinoff_prover]
pub proof fn extension(a:Trace,b:Trace,e:Step)
    requires shape(a),enabled(a,e),b==step_after(a,e),
    ensures extends(a,b),
{
    assert forall|n:int| 0<=n<a.nodes.len() implies (#[trigger] a.nodes[n]).state.config==b.nodes[n].state.config
        && a.nodes[n].choices.is_prefix_of(b.nodes[n].choices) by {
        if n!=actor(a,e) {assert(a.nodes[n]==b.nodes[n]);}
        else {match e {Step::Start(_)=>{},Step::Save{..}=>{},_=>{}}}
    }
}
pub proof fn lift_choice(a:Trace,b:Trace,v:u64,n:u64,term:u64)
    requires extends(a,b),choice_for(a,v,n,term),
    ensures choice_for(b,v,n,term),
{
    let i=choose|i:int| 0<=i<a.nodes[v as int].choices.len() && a.nodes[v as int].choices[i]==Choice{node:v,term,candidate:n};
    assert(b.nodes[v as int].choices[i]==a.nodes[v as int].choices[i]);
}
#[verifier::spinoff_prover]
pub proof fn actor_local(a:Trace,b:Trace,e:Step)
    requires inv(a),shape(a),enabled(a,e),b==step_after(a,e),
    ensures local(b.nodes[actor(a,e)]),
{
    let n=actor(a,e);let o=a.nodes[n];let u=b.nodes[n];
    assert(local(o));
    match e {
        Step::Start(_)=>{
            if let Campaign::Save(w)=start_result(o.state) {
                crate::vote_core::history_advance(o.state,u.state,o.choices,o.base);
                crate::vote_core::history_append(u.state,o.choices,o.base);
                assert(u.choices.last()==choice(u.state));
                assert(u.choices.contains(choice(u.state)));
            }
        },
        Step::Save{node,success}=>{
            let p=a.pending[n].unwrap();assert(o.state.pending==Some(p.write));
            if success {if let Action::Reply(r)=p.write.action {if r.granted {
                crate::vote_core::history_append(o.state,o.choices,o.base);
            }}}
        },
        Step::Receive(d)=>{crate::vote_core::history_advance(o.state,u.state,o.choices,o.base);},
        Step::Deliver(i)=>{
            let r=a.replies[i as int];let d=r.request.unwrap();
            if collect_result(o.state,o.votes,context(a,d),reply_result(r))==Collect::Counted {
                crate::vote_core::bounded_votes(u.votes,o.state.config.members);
            }
            crate::vote_core::history_advance(o.state,u.state,o.choices,o.base);
        },
        Step::Error(d)=>{crate::vote_core::history_advance(o.state,u.state,o.choices,o.base);},
        Step::Promote(_)=>{},
    }
}
#[verifier::spinoff_prover]
pub proof fn preserve(a:Trace,b:Trace,e:Step)
    requires inv(a),shape(a),enabled(a,e),b==step_after(a,e),
    ensures inv(b),
{
    let at=actor(a,e);extension(a,b,e);actor_local(a,b,e);
    assert(0<=at<a.nodes.len());
    assert forall|n:int| 0<=n<b.nodes.len() implies local(#[trigger] b.nodes[n]) && pending_ok(b,n) by {
        if n!=at {assert(b.nodes[n]==a.nodes[n]);assert(b.pending[n]==a.pending[n]);}
        else {match e {Step::Start(_)=>{},Step::Receive(_)=>{},Step::Save{..}=>{},Step::Promote(_)=>{},Step::Deliver(_)=>{},Step::Error(_)=>{}}}
    }
    assert forall|i:int| 0<=i<b.sent.len() implies sent_ok(b,i) by {
        if i<a.sent.len() {assert(b.sent[i]==a.sent[i]);let s=a.sent[i];lift_choice(a,b,s.node,s.node,s.payload.request.term);}
        else {
            assert(e is Save);let s=b.sent[i];let p=a.pending[at].unwrap();
            assert(p.write.action==Action::Send(s.payload));
            assert(a.nodes[at].choices.contains(Choice{node:s.node,term:s.payload.request.term,candidate:s.node}));
            assert(choice_for(a,s.node,s.node,s.payload.request.term));lift_choice(a,b,s.node,s.node,s.payload.request.term);
        }
    }
    assert forall|i:int| 0<=i<b.replies.len() implies returned_ok(b,i) by {
        if i<a.replies.len() {assert(b.replies[i]==a.replies[i]);let r=a.replies[i];if r.payload.granted {lift_choice(a,b,r.node,r.payload.candidate,r.payload.term);}}
        else {
            assert(e is Save);let r=b.replies[i];let p=a.pending[at].unwrap();
            assert(p.write.action==Action::Reply(r.payload));
            if r.payload.granted {
                assert(b.nodes[at].choices.last()==Choice{node:r.node,term:r.payload.term,candidate:r.payload.candidate});
                assert(choice_for(b,r.node,r.payload.candidate,r.payload.term));
            }
        }
    }
    assert(all_sourced(b));assert(choice_sourced(b));assert(elections_justified(b));
}
}
