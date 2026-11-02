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
    &&& (forall|i:int| 0<=i<t.frames.len() ==> (#[trigger] t.frames[i]).change.before.config==Config{node:t.frames[i].change.node,members:t.nodes.len() as u64})
}
pub proof fn network_initial(t:Trace)
    requires boot(t), ensures network_inv(t),
{
    assert forall|a:int| 0<=a<t.nodes.len() implies local(#[trigger] t.nodes[a]) && proposed_ok(t,a) by {
        assert(t.nodes[a].choices==Seq::<Choice>::empty());
        assert(t.nodes[a].votes==Set::<u64>::empty());
    }
}
pub proof fn append_history(o:Observation,c:Choice)
    requires local(o), c.node==o.state.config.node,c.term==o.state.state.term,o.state.state.voted_for==Some(c.candidate),
    ensures unique(o.choices.push(c)),
{
    let h=o.choices;let next=h.push(c);
    assert forall|i:int,j:int| 0<=i<next.len() && 0<=j<next.len() && (#[trigger] next[i]).node==(#[trigger] next[j]).node && next[i].term==next[j].term implies next[i].candidate==next[j].candidate by {
        if i<h.len() && j<h.len() {assert(h[i].candidate==h[j].candidate);}
        else if i<h.len() {assert(h[i].term==o.state.state.term);}
        else if j<h.len() {assert(h[j].term==o.state.state.term);}
    }
}
pub proof fn local_receive(o:Observation,r:Request)
    requires local(o), ensures local(receiver(o,r)),
{
    let out=receive_result(o.state,r);
    match out { Receive::Save(w)=>{},_=>{} }
}
pub proof fn local_start(o:Observation)
    requires local(o), ensures local(starter(o)),
{
    let out=start_result(o.state);
    if out is Save {
        let a=start_after(o.state,out);let c=choice(a);
        assert forall|i:int,j:int| 0<=i<starter(o).choices.len() && 0<=j<starter(o).choices.len() && (#[trigger] starter(o).choices[i]).node==(#[trigger] starter(o).choices[j]).node && starter(o).choices[i].term==starter(o).choices[j].term implies starter(o).choices[i].candidate==starter(o).choices[j].candidate by {
            if i<o.choices.len() && j<o.choices.len() {assert(o.choices[i].candidate==o.choices[j].candidate);}
        }
    }
}
pub proof fn local_finish(o:Observation,n:Completion)
    requires local(o), ensures local(finisher(o,n)),
{
    if let Some(Action::Reply(r))=complete_result(o.state,n) {
        if r.granted {append_history(o,Choice{node:r.node,term:r.term,candidate:r.candidate});}
    }
}
pub proof fn local_collect(o:Observation,c:RpcContext,r:RpcResult)
    requires local(o), ensures local(collector(o,c,r)),
{ }
pub proof fn local_promote(o:Observation)
    requires local(o), ensures local(Observation{state:promotion_after(o.state,o.votes),..o}),
{ }
pub proof fn network_step(before:Trace,after:Trace,event:Step)
    requires network_inv(before),shape(before),enabled(before,event),after==step_after(before,event),
    ensures network_inv(after),
{ assert(false); }
pub proof fn network_conclusion(t:Trace)
    requires network_inv(t),shape(t),
    ensures sourced(t),choice_sourced(t),elections_justified(t),historical_safety(t),
{
    assert forall|i:int,j:int| #![trigger changes(t)[i],changes(t)[j]]
        0<=i<changes(t).len() && 0<=j<changes(t).len() && election_observation::elects(changes(t)[i]) && election_observation::elects(changes(t)[j]) && changes(t)[i].after.state.term==changes(t)[j].after.state.term implies changes(t)[i].node==changes(t)[j].node by {
        let f=t.frames[i];let g=t.frames[j];
        assert(f.change==changes(t)[i] && g.change==changes(t)[j]);
        intersect_majorities(f.change.before.config,g.change.before.config,f.votes,g.votes);
        let v=choose|v:u64| f.votes.contains(v) && g.votes.contains(v);
        assert(choice_for(t,v,f.change.node,f.change.after.state.term));
        assert(choice_for(t,v,g.change.node,g.change.after.state.term));
        let h=t.nodes[v as int].choices;
        let a=choose|a:int| 0<=a<h.len() && h[a]==Choice{node:v,term:f.change.after.state.term,candidate:f.change.node};
        let b=choose|b:int| 0<=b<h.len() && h[b]==Choice{node:v,term:g.change.after.state.term,candidate:g.change.node};
        assert(unique(h));assert(h[a].candidate==h[b].candidate);
    }
}
pub proof fn intersect_majorities(c:Config,d:Config,a:Set<u64>,b:Set<u64>)
    requires majority(c,a),majority(d,b),c.members==d.members,
    ensures exists|v:u64| a.contains(v) && b.contains(v),
{
    Core::bounded(a+b,c.members);
    if !(exists|v:u64| a.contains(v) && b.contains(v)) {
        assert(a.disjoint(b));vstd::set_lib::lemma_set_disjoint_lens(a,b);
        assert(c.members <= 2*(c.members/2)+1) by (nonlinear_arith);
        assert(false);
    }
}
}
