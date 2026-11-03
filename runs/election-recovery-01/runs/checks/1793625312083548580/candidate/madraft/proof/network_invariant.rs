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
#[verifier::spinoff_prover]
pub proof fn preserve(a:Trace,b:Trace,e:Step)
    requires inv(a),shape(a),enabled(a,e),b==step_after(a,e),
    ensures inv(b),
{
    let at=actor(a,e);
    assert(0<=at<a.nodes.len());
    assert forall|n:int| 0<=n<b.nodes.len() implies local(#[trigger] b.nodes[n]) && pending_ok(b,n) by {
        if n!=at {assert(b.nodes[n]==a.nodes[n]);assert(b.pending[n]==a.pending[n]);}
        else {match e {Step::Start(_)=>{},Step::Receive(_)=>{},Step::Save{..}=>{},Step::Promote(_)=>{},Step::Deliver(_)=>{},Step::Error(_)=>{}}}
    }
    assert forall|i:int| 0<=i<b.sent.len() implies sent_ok(b,i) by {}
    assert forall|i:int| 0<=i<b.replies.len() implies returned_ok(b,i) by {}
    assert(all_sourced(b));assert(choice_sourced(b));assert(elections_justified(b));
}
}
