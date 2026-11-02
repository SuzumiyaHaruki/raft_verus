use crate::{vote_api::*,vote_core::Core,vote_network::*,election_observation};
use vstd::prelude::*;
verus! {
broadcast use {vstd::seq_lib::group_seq_properties,vstd::seq_lib::group_seq_lib_default,vstd::set::group_set_axioms};
pub open spec fn has_choice(o:Observation,c:Choice) -> bool {o.choices.contains(c)}
pub open spec fn local(o:Observation) -> bool {
    let v=o.state;
    &&& valid_state(v.config,v.state)
    &&& o.base.is_none()
    &&& unique(o.choices)
    &&& (forall|i:int| 0<=i<o.choices.len() ==> {
        let c=#[trigger] o.choices[i];
        c.node==v.config.node && c.term<=v.state.term && (c.term==v.state.term ==> v.state.voted_for==Some(c.candidate))
    })
    &&& o.votes.finite()
    &&& (forall|n:u64| #[trigger] o.votes.contains(n) ==> n<v.config.members)
    &&& (o.votes!=Set::<u64>::empty() ==> !v.halted && v.self_saved && v.round==Some(v.state.term) && (v.role==Role::Candidate || v.role==Role::Leader))
    &&& (v.self_saved ==> v.state.voted_for==Some(v.config.node) && has_choice(o,Choice{node:v.config.node,term:v.state.term,candidate:v.config.node}))
    &&& (v.pending.is_some() ==> {
        let w=v.pending.unwrap();
        &&& !v.halted && valid_record(v.config,w.record) && w.record.state==v.state && w.record.op+1==v.next
        &&& match w.action {
            Action::Send(s) => s.config==v.config && s.request==Request{term:v.state.term,candidate:v.config.node,log:v.state.log} && valid_request(v.config,s.request) && v.role==Role::Candidate && v.round==Some(v.state.term) && !v.self_saved && has_choice(o,Choice{node:v.config.node,term:v.state.term,candidate:v.config.node}),
            Action::Reply(r) => r.node==v.config.node && r.term==v.state.term && (r.granted ==> v.state.voted_for==Some(r.candidate)),
            Action::Persisted => true,
        }
    })
}
pub open spec fn proposed_ok(t:Trace,a:int) -> bool {
    let o=t.nodes[a];
    &&& t.pending[a].is_some()==o.state.pending.is_some()
    &&& (t.pending[a].is_some() ==> {
        let p=t.pending[a].unwrap();
        &&& o.state.pending==Some(p.write)
        &&& match p.write.action {
            Action::Reply(r) => p.request.is_some() && routed(t,p.request.unwrap()) && p.request.unwrap().peer==a && p.write.request==Some(t.sent[p.request.unwrap().send as int].payload.request) && r.candidate==t.sent[p.request.unwrap().send as int].payload.request.candidate,
            _ => p.request.is_none(),
        }
    })
}
pub open spec fn sent_ok(t:Trace,i:int) -> bool {
    let s=t.sent[i];
    &&& s.node<t.nodes.len()
    &&& s.payload.config==t.nodes[s.node as int].state.config
    &&& valid_request(s.payload.config,s.payload.request)
    &&& s.payload.request.candidate==s.node
    &&& s.notice.success && s.notice.op==s.write.record.op
    &&& s.write.record.state.term==s.payload.request.term && s.write.record.state.voted_for==Some(s.node)
    &&& choice_for(t,s.node,s.node,s.payload.request.term)
}
pub open spec fn reply_ok(t:Trace,i:int) -> bool {
    let r=t.replies[i];
    &&& r.request.is_some() && routed(t,r.request.unwrap())
    &&& r.node==r.request.unwrap().peer && r.payload.node==r.node
    &&& r.payload.candidate==t.sent[r.request.unwrap().send as int].payload.request.candidate
    &&& (r.payload.granted ==> choice_for(t,r.node,r.payload.candidate,r.payload.term))
}
pub open spec fn vote_sources(t:Trace) -> bool {
    forall|a:int,v:u64| 0<=a<t.nodes.len() && #[trigger] t.nodes[a].votes.contains(v) ==> if v==a {
        self_evidence(t,a,t.nodes[a].state.state.term)
    } else {remote_evidence(t,a,t.nodes[a].state.state.term,v)}
}
pub open spec fn network_inv(t:Trace) -> bool {
    &&& shape(t)
    &&& (forall|a:int| 0<=a<t.nodes.len() ==> local(#[trigger] t.nodes[a]) && proposed_ok(t,a))
    &&& (forall|i:int| 0<=i<t.sent.len() ==> sent_ok(t,i))
    &&& (forall|i:int| 0<=i<t.replies.len() ==> reply_ok(t,i))
    &&& vote_sources(t)
    &&& choice_sourced(t)
    &&& elections_justified(t)
}
pub proof fn network_initial(t:Trace)
    requires boot(t), ensures network_inv(t),
{
    assert forall|a:int| 0<=a<t.nodes.len() implies local(#[trigger] t.nodes[a]) && proposed_ok(t,a) by {
        assert(t.nodes[a].choices==Seq::<Choice>::empty());
        assert(t.nodes[a].votes==Set::<u64>::empty());
    }
}
pub proof fn network_step(before:Trace,after:Trace,event:Step)
    requires network_inv(before),shape(before),enabled(before,event),after==step_after(before,event),
    ensures network_inv(after),
{ assert(false); }
pub proof fn network_conclusion(t:Trace)
    requires network_inv(t),shape(t),
    ensures sourced(t),choice_sourced(t),elections_justified(t),historical_safety(t),
{ assert(false); }
}
