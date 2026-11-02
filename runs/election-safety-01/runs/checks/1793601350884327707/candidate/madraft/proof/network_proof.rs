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
pub open spec fn extends(b:Trace,t:Trace) -> bool {
    &&& b.nodes.len()==t.nodes.len()
    &&& b.sent.is_prefix_of(t.sent) && b.replies.is_prefix_of(t.replies) && b.delivered.is_prefix_of(t.delivered)
    &&& (forall|n:int| 0<=n<b.nodes.len() ==> (#[trigger] b.nodes[n]).state.config==t.nodes[n].state.config && b.nodes[n].choices.is_prefix_of(t.nodes[n].choices))
}
pub proof fn keep_choice(b:Trace,t:Trace,v:u64,c:u64,term:u64)
    requires extends(b,t),choice_for(b,v,c,term), ensures choice_for(t,v,c,term),
{
    let i=choose|i:int| 0<=i<b.nodes[v as int].choices.len() && b.nodes[v as int].choices[i]==Choice{node:v,term,candidate:c};
    assert(t.nodes[v as int].choices[i]==b.nodes[v as int].choices[i]);
}
pub proof fn keep_source(b:Trace,t:Trace,n:int,v:u64,term:u64)
    requires extends(b,t),0<=n<b.nodes.len(),if v==n {self_evidence(b,n,term)} else {remote_evidence(b,n,term,v)},
    ensures if v==n {self_evidence(t,n,term)} else {remote_evidence(t,n,term,v)},
{
    if v==n {
        let i=choose|i:int| 0<=i<b.sent.len() && b.sent[i].node==n && b.sent[i].notice.success && b.sent[i].notice.op==b.sent[i].write.record.op && b.sent[i].payload.config==b.nodes[n].state.config && b.sent[i].payload.request.term==term && b.sent[i].payload.request.candidate==n && b.sent[i].write.record.state.term==term && b.sent[i].write.record.state.voted_for==Some(n as u64);
        assert(t.sent[i]==b.sent[i]);
    } else {
        let j=choose|j:int| 0<=j<b.delivered.len() && {
            let i=#[trigger] b.delivered[j];
            &&& i<b.replies.len() && b.replies[i as int].request.is_some()
            &&& {let r=b.replies[i as int];let d=r.request.unwrap();
                d.send<b.sent.len() && b.sent[d.send as int].node==n && b.sent[d.send as int].payload.request.candidate==n && b.sent[d.send as int].payload.request.term==term && d.peer==v && r.node==v && r.payload.node==v && r.payload.candidate==n && r.payload.term==term && r.payload.granted}
        };
        let i=#[trigger] b.delivered[j];let d=b.replies[i as int].request.unwrap();
        assert(t.delivered[j]==i && t.replies[i as int]==b.replies[i as int] && t.sent[d.send as int]==b.sent[d.send as int]);
    }
}
pub proof fn source_choices(t:Trace)
    requires shape(t),vote_sources(t),
        forall|i:int| 0<=i<t.sent.len() ==> sent_ok(t,i),
        forall|i:int| 0<=i<t.replies.len() ==> reply_ok(t,i),
    ensures choice_sourced(t),
{
    assert forall|a:int,v:u64| 0<=a<t.nodes.len() && #[trigger] t.nodes[a].votes.contains(v) implies choice_for(t,v,a as u64,t.nodes[a].state.state.term) by {
        let term=t.nodes[a].state.state.term;
        if v==a {
            let i=choose|i:int| 0<=i<t.sent.len() && t.sent[i].node==a && t.sent[i].notice.success && t.sent[i].notice.op==t.sent[i].write.record.op && t.sent[i].payload.config==t.nodes[a].state.config && t.sent[i].payload.request.term==term && t.sent[i].payload.request.candidate==a && t.sent[i].write.record.state.term==term && t.sent[i].write.record.state.voted_for==Some(a as u64);
            assert(sent_ok(t,i));
        } else {
            let j=choose|j:int| 0<=j<t.delivered.len() && {
                let i=#[trigger] t.delivered[j];
                &&& i<t.replies.len() && t.replies[i as int].request.is_some()
                &&& {let r=t.replies[i as int];let d=r.request.unwrap();
                    d.send<t.sent.len() && t.sent[d.send as int].node==a && t.sent[d.send as int].payload.request.candidate==a && t.sent[d.send as int].payload.request.term==term && d.peer==v && r.node==v && r.payload.node==v && r.payload.candidate==a && r.payload.term==term && r.payload.granted}
            };
            assert(reply_ok(t,t.delivered[j] as int));
        }
    }
}
pub proof fn network_step(before:Trace,after:Trace,event:Step)
    requires network_inv(before),shape(before),enabled(before,event),after==step_after(before,event),
    ensures network_inv(after),
{
    let a=actor(before,event);let o=before.nodes[a];let t=after;let b=before;
    assert(0<=a<b.nodes.len());
    match event {
        Step::Promote(_) => local_promote(o),
        Step::Start(_) => local_start(o),
        Step::Receive(d) => local_receive(o,b.sent[d.send as int].payload.request),
        Step::Save{node,success} => local_finish(o,Completion{op:b.pending[node as int].unwrap().write.record.op,success}),
        Step::Deliver(i) => {let r=b.replies[i as int];local_collect(o,context(b,r.request.unwrap()),reply_result(r));},
        Step::Error(d) => local_collect(o,context(b,d),RpcResult::Error),
    }
    assert forall|n:int| 0<=n<t.nodes.len() implies local(#[trigger] t.nodes[n]) && proposed_ok(t,n) by {
        if n!=a {assert(t.nodes[n]==b.nodes[n]);}
        else {match event {
            Step::Promote(_) | Step::Start(_) => {},
            Step::Receive(d) => {assert(sent_ok(b,d.send as int));},
            Step::Save{node,success} => {assert(proposed_ok(b,node as int));},
            Step::Deliver(i) => {},Step::Error(_) => {},
        }}
    }
    assert(extends(b,t));
    assert forall|i:int| 0<=i<t.sent.len() implies sent_ok(t,i) by {
        if i<b.sent.len() {let s=b.sent[i];keep_choice(b,t,s.node,s.node,s.payload.request.term);}
        else {match event {Step::Save{node,success} => {
            let p=b.pending[node as int].unwrap();assert(p.write==o.state.pending.unwrap());
            assert(has_choice(o,Choice{node:node as u64,term:o.state.state.term,candidate:node as u64}));
            assert(choice_for(b,node as u64,node as u64,o.state.state.term));
            keep_choice(b,t,node as u64,node as u64,o.state.state.term);
        },_=>{}}}
    }
    assert forall|i:int| 0<=i<t.replies.len() implies reply_ok(t,i) by {
        if i<b.replies.len() {let r=b.replies[i];if r.payload.granted {keep_choice(b,t,r.node,r.payload.candidate,r.payload.term);}}
        else {match event {Step::Save{node,success} => {
            let r=t.replies[i];assert(proposed_ok(b,node as int));
            if r.payload.granted {
                assert(t.nodes[a].choices.last()==Choice{node:r.node,term:r.payload.term,candidate:r.payload.candidate});
                assert(choice_for(t,r.node,r.payload.candidate,r.payload.term));
            }
        },_=>{}}}
    }
    assert forall|n:int,v:u64| 0<=n<t.nodes.len() && #[trigger] t.nodes[n].votes.contains(v) implies if v==n {self_evidence(t,n,t.nodes[n].state.state.term)} else {remote_evidence(t,n,t.nodes[n].state.state.term,v)} by {
        let term=t.nodes[n].state.state.term;
        if n!=a || (b.nodes[n].votes.contains(v) && term==b.nodes[n].state.state.term) {keep_source(b,t,n,v,term);}
        else {match event {
            Step::Save{node,success} => {assert(v==n);assert(t.sent[b.sent.len() as int].node==n);assert(self_evidence(t,n,term));},
            Step::Deliver(i) => {
                let r=b.replies[i as int];let d=r.request.unwrap();
                assert(reply_ok(b,i as int));assert(sent_ok(b,d.send as int));
                assert(v==d.peer && t.delivered[b.delivered.len() as int]==i);
                assert(remote_evidence(t,n,term,v));
            },_=>{},
        }}
    }
    source_choices(t);
    assert forall|i:int| 0<=i<t.frames.len() implies {
        let f=#[trigger] t.frames[i];
        election_observation::elects(f.change) ==> {
            &&& f.change.node<t.nodes.len()
            &&& promotion_allowed(f.change.before,f.votes)
            &&& f.change.after==promotion_after(f.change.before,f.votes)
            &&& forall|v:u64| #[trigger] f.votes.contains(v) ==> choice_for(t,v,f.change.node,f.change.after.state.term)
        }
    } by {
        let f=t.frames[i];
        if i<b.frames.len() {
            assert(f==b.frames[i]);
            assert forall|v:u64| election_observation::elects(f.change) && #[trigger] f.votes.contains(v) implies choice_for(t,v,f.change.node,f.change.after.state.term) by {keep_choice(b,t,v,f.change.node,f.change.after.state.term);}
        } else {
            match event {Step::Promote(_) => {},_=>{assert(!election_observation::elects(f.change));}}
            assert forall|v:u64| election_observation::elects(f.change) && #[trigger] f.votes.contains(v) implies choice_for(t,v,f.change.node,f.change.after.state.term) by {keep_choice(b,t,v,f.change.node,f.change.after.state.term);}
        }
    }
    assert forall|i:int| 0<=i<t.frames.len() implies (#[trigger] t.frames[i]).change.before.config==Config{node:t.frames[i].change.node,members:t.nodes.len() as u64} by {
        if i<b.frames.len() {assert(t.frames[i]==b.frames[i]);}
    }
}
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
