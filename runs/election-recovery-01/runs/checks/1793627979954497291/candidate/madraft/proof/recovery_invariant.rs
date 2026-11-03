//! Cross-instance ledger locks over actual retained writes and recovery calls.
use crate::{vote_api::*,recovery_network::*,vote_network as old};
use vstd::prelude::*;
#[path="network_invariant.rs"] pub mod basis;
#[path="recovery_source.rs"] pub mod source;
verus! {
broadcast use {vstd::seq_lib::group_seq_properties,vstd::set::group_set_axioms};
pub open spec fn locked(v:crate::vote_api::View,h:Seq<Choice>) -> bool {
    forall|i:int| 0<=i<h.len() && (#[trigger] h[i]).node==v.config.node ==> h[i].term<=v.state.term
        && (h[i].term==v.state.term ==> v.state.voted_for==Some(h[i].candidate))
}
pub open spec fn disk_lock(t:Trace,n:int) -> bool {
    &&& (t.io[n].is_some() && t.io[n].unwrap().result==Some(true) ==> t.images[n].is_some()
        && t.images[n].unwrap().record==t.io[n].unwrap().proposed.write.record)
    &&& (!t.live[n] || t.ready[n] ==> t.images[n].is_some()
        && t.images[n].unwrap().record.state==t.net.nodes[n].state.state
        && t.images[n].unwrap().record.op+1==t.net.nodes[n].state.next)
}
pub open spec fn sent_ok(t:Trace,i:int) -> bool {
    let s=t.net.sent[i];
    &&& s.node<t.net.nodes.len() && s.payload.config==t.net.nodes[s.node as int].state.config
    &&& valid_request(s.payload.config,s.payload.request) && s.payload.request.candidate==s.node
    &&& s.notice.success && s.notice.op==s.write.record.op && s.write.action==Action::Send(s.payload)
    &&& s.write.record.state.term==s.payload.request.term && s.write.record.state.voted_for==Some(s.node)
    &&& choice_for(t,s.node,s.node,s.payload.request.term)
}
pub open spec fn returned_ok(t:Trace,i:int) -> bool {
    let r=t.net.replies[i];
    &&& r.request.is_some() && old::routed(t.net,r.request.unwrap())
    &&& r.node==r.request.unwrap().peer && r.payload.node==r.node
    &&& r.payload.candidate==t.net.sent[r.request.unwrap().send as int].payload.request.candidate
    &&& r.payload.granted ==> r.payload.term==t.net.sent[r.request.unwrap().send as int].payload.request.term
        && choice_for(t,r.node,r.payload.candidate,r.payload.term)
}
pub open spec fn inv(t:Trace) -> bool {
    &&& t.net.nodes.len()<=usize::MAX
    &&& source::source_inv(t) && unique(t.choices)
    &&& forall|n:int| 0<=n<t.net.nodes.len() ==> {
        let o=#[trigger] t.net.nodes[n];
        &&& basis::local(o) && locked(o.state,t.choices) && disk_lock(t,n)
        &&& (o.state.pending.is_some() ==> o.state.pending.unwrap().record.op+1==o.state.next)
        &&& (t.live[n] ==> basis::pending_ok(t.net,n))
        &&& forall|i:int| 0<=i<o.choices.len() ==> t.choices.contains(#[trigger] o.choices[i])
    }
    &&& forall|i:int| 0<=i<t.net.sent.len() ==> sent_ok(t,i)
    &&& forall|i:int| 0<=i<t.net.replies.len() ==> returned_ok(t,i)
    &&& forall|i:int| 0<=i<t.frames.len() ==> {
        let f=#[trigger] t.frames[i];f.node<t.net.nodes.len() && f.before.state.config==t.net.nodes[f.node as int].state.config
            && f.after.state.config==f.before.state.config
    }
    &&& provenance(t)
}
pub proof fn initialize(t:Trace)
    requires boot(t),t.net.nodes.len()<=usize::MAX,ensures inv(t),
{
    source::initialize(t);basis::initialize(t.net);
    assert forall|n:int| 0<=n<t.net.nodes.len() implies {
        let o=#[trigger] t.net.nodes[n];
        &&& basis::local(o) && locked(o.state,t.choices) && disk_lock(t,n)
        &&& (o.state.pending.is_some() ==> o.state.pending.unwrap().record.op+1==o.state.next)
        &&& (t.live[n] ==> basis::pending_ok(t.net,n))
        &&& forall|i:int| 0<=i<o.choices.len() ==> t.choices.contains(#[trigger] o.choices[i])
    } by {assert(t.live[n] && t.epochs[n]==0 && !t.ready[n]);assert(t.io[n].is_none());}
}
#[verifier::spinoff_prover]
#[verifier::rlimit(1)]
pub proof fn conclude(t:Trace)
    requires inv(t),shape(t),
    ensures recovered_from_own_write(t),unique(t.choices),provenance(t),historical_safety(t),
{
    assert forall|i:int,j:int| #![trigger t.frames[i],t.frames[j]] 0<=i<t.frames.len() && 0<=j<t.frames.len()
        && elects(t.frames[i]) && elects(t.frames[j]) && t.frames[i].after.state.state.term==t.frames[j].after.state.state.term
        implies t.frames[i].node==t.frames[j].node by {
        let a=t.frames[i];let b=t.frames[j];
        basis::intersect(a.before.state.config,b.before.state.config,a.before.votes,b.before.votes);
        let v=choose|v:u64| a.before.votes.contains(v) && b.before.votes.contains(v);
        let term=a.after.state.state.term;
        assert(choice_for(t,v,a.node as u64,term));assert(choice_for(t,v,b.node as u64,term));
        let x=choose|x:int| 0<=x<t.choices.len() && t.choices[x]==Choice{node:v,term,candidate:a.node as u64};
        let y=choose|y:int| 0<=y<t.choices.len() && t.choices[y]==Choice{node:v,term,candidate:b.node as u64};
        assert(t.choices[x].candidate==t.choices[y].candidate);
    }
}
}
