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
    &&& forall|i:int| 0<=i<t.frames.len() ==> {
        let f=#[trigger] t.frames[i];f.change.node<t.nodes.len()
            && f.change.before.config==t.nodes[f.change.node as int].state.config
            && f.change.after.config==f.change.before.config
    }
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
pub proof fn lift_evidence(a:Trace,b:Trace,n:int,v:u64,term:u64)
    requires extends(a,b),0<=n<a.nodes.len(),
        if v==n {self_evidence(a,n,term)} else {remote_evidence(a,n,term,v)},
    ensures if v==n {self_evidence(b,n,term)} else {remote_evidence(b,n,term,v)},
{
    if v==n {
        let i=choose|i:int| 0<=i<a.sent.len() && a.sent[i].node==n
            && a.sent[i].notice.success && a.sent[i].notice.op==a.sent[i].write.record.op
            && a.sent[i].payload.config==a.nodes[n].state.config
            && a.sent[i].payload.request.term==term && a.sent[i].payload.request.candidate==n
            && a.sent[i].write.record.state.term==term && a.sent[i].write.record.state.voted_for==Some(n as u64);
        assert(b.sent[i]==a.sent[i]);
    } else {
        let j=choose|j:int| 0<=j<a.delivered.len() && {
            let i=a.delivered[j];
            i<a.replies.len() && a.replies[i as int].request.is_some() && {
                let r=a.replies[i as int];let d=r.request.unwrap();
                d.send<a.sent.len() && a.sent[d.send as int].node==n && a.sent[d.send as int].payload.request.candidate==n
                && a.sent[d.send as int].payload.request.term==term && d.peer==v && r.node==v && r.payload.node==v
                && r.payload.candidate==n && r.payload.term==term && r.payload.granted
            }
        };
        let i=a.delivered[j];assert(b.delivered[j]==i);assert(b.replies[i as int]==a.replies[i as int]);
        let d=a.replies[i as int].request.unwrap();assert(b.sent[d.send as int]==a.sent[d.send as int]);
    }
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
    assert(local(a.nodes[at]) && pending_ok(a,at));
    assert(0<=at<a.nodes.len());
    assert forall|n:int| 0<=n<b.nodes.len() implies local(#[trigger] b.nodes[n]) && pending_ok(b,n) by {
        if n!=at {assert(b.nodes[n]==a.nodes[n]);assert(b.pending[n]==a.pending[n]);}
        else {match e {Step::Start(_)=>{},Step::Receive(_)=>{},Step::Save{..}=>{},Step::Promote(_)=>{},Step::Deliver(_)=>{},Step::Error(_)=>{}}}
    }
    assert forall|i:int| 0<=i<b.sent.len() implies sent_ok(b,i) by {
        if i<a.sent.len() {assert(sent_ok(a,i));assert(b.sent[i]==a.sent[i]);let s=a.sent[i];lift_choice(a,b,s.node,s.node,s.payload.request.term);}
        else {
            assert(e is Save);let s=b.sent[i];let p=a.pending[at].unwrap();
            assert(p.write.action==Action::Send(s.payload));
            assert(a.nodes[at].choices.contains(Choice{node:s.node,term:s.payload.request.term,candidate:s.node}));
            assert(choice_for(a,s.node,s.node,s.payload.request.term));lift_choice(a,b,s.node,s.node,s.payload.request.term);
        }
    }
    assert forall|i:int| 0<=i<b.replies.len() implies returned_ok(b,i) by {
        if i<a.replies.len() {assert(returned_ok(a,i));assert(b.replies[i]==a.replies[i]);let r=a.replies[i];if r.payload.granted {lift_choice(a,b,r.node,r.payload.candidate,r.payload.term);}}
        else {
            assert(e is Save);let r=b.replies[i];let p=a.pending[at].unwrap();
            assert(p.write.action==Action::Reply(r.payload));
            if r.payload.granted {
                assert(b.nodes[at].choices.last()==Choice{node:r.node,term:r.payload.term,candidate:r.payload.candidate});
                assert(choice_for(b,r.node,r.payload.candidate,r.payload.term));
            }
        }
    }
    assert forall|n:int,v:u64| 0<=n<b.nodes.len() && #[trigger] b.nodes[n].votes.contains(v)
        implies if v==n {self_evidence(b,n,b.nodes[n].state.state.term)} else {remote_evidence(b,n,b.nodes[n].state.state.term,v)} by {
        if a.nodes[n].votes.contains(v) && a.nodes[n].state.state.term==b.nodes[n].state.state.term {
            lift_evidence(a,b,n,v,b.nodes[n].state.state.term);
        } else {
            match e {
                Step::Save{node,success}=>{
                    assert(n==at && v==n && success);
                    let i=a.sent.len() as int;let emitted=b.sent[i];
                    assert(emitted.node==n && emitted.payload.request.term==b.nodes[n].state.state.term);
                    assert(self_evidence(b,n,b.nodes[n].state.state.term));
                },
                Step::Deliver(i)=>{
                    let r=a.replies[i as int];let d=r.request.unwrap();let ss=a.sent[d.send as int];
                    assert(n==at && v==d.peer);assert(returned_ok(a,i as int));assert(sent_ok(a,d.send as int));
                    assert(b.delivered[a.delivered.len() as int]==i);
                    assert(remote_evidence(b,n,b.nodes[n].state.state.term,v));
                },_=>{assert(false);},
            }
        }
    }
    assert forall|n:int,v:u64| 0<=n<b.nodes.len() && #[trigger] b.nodes[n].votes.contains(v)
        implies choice_for(b,v,n as u64,b.nodes[n].state.state.term) by {
        if a.nodes[n].votes.contains(v) && a.nodes[n].state.state.term==b.nodes[n].state.state.term {
            assert(choice_for(a,v,n as u64,a.nodes[n].state.state.term));lift_choice(a,b,v,n as u64,b.nodes[n].state.state.term);
        } else {match e {
            Step::Save{node,success}=>{assert(v==n && n==at);let ss=b.sent[a.sent.len() as int];assert(sent_ok(b,a.sent.len() as int));},
            Step::Deliver(i)=>{let r=a.replies[i as int];assert(returned_ok(a,i as int));assert(v==r.node && n==at && r.payload.granted);lift_choice(a,b,v,n as u64,r.payload.term);},
            _=>{assert(false);},
        }}
    }
    assert(elections_justified(b));
}
pub proof fn intersect(c:Config,d:Config,x:Set<u64>,y:Set<u64>)
    requires c.members==d.members,majority(c,x),majority(d,y),
    ensures exists|v:u64| x.contains(v) && y.contains(v),
{
    let union=x.union(y);
    crate::vote_core::bounded_votes(union,c.members);
    if !(exists|v:u64| x.contains(v) && y.contains(v)) {
        assert(x.disjoint(y));vstd::set_lib::lemma_set_disjoint_lens(x,y);
        assert(union.len()==x.len()+y.len());
        assert(c.members as nat <= 2*(c.members/2) as nat+1);
        assert(false);
    }
}
#[verifier::spinoff_prover]
pub proof fn conclude(t:Trace)
    requires inv(t),shape(t),
    ensures sourced(t),choice_sourced(t),elections_justified(t),historical_safety(t),
{
    assert(sourced(t));
    let ch=changes(t);
    assert forall|i:int,j:int| #![trigger ch[i],ch[j]]
        0<=i<ch.len() && 0<=j<ch.len() && election_observation::elects(ch[i]) && election_observation::elects(ch[j])
        && ch[i].after.state.term==ch[j].after.state.term implies ch[i].node==ch[j].node by {
        let a=t.frames[i];let b=t.frames[j];assert(ch[i]==a.change && ch[j]==b.change);
        assert(promotion_allowed(a.change.before,a.votes));assert(promotion_allowed(b.change.before,b.votes));
        assert(a.change.before.config.members==b.change.before.config.members);
        intersect(a.change.before.config,b.change.before.config,a.votes,b.votes);
        let v=choose|v:u64| a.votes.contains(v) && b.votes.contains(v);
        let term=a.change.after.state.term;
        assert(choice_for(t,v,a.change.node,term));assert(choice_for(t,v,b.change.node,term));
        let h=t.nodes[v as int].choices;
        let x=choose|x:int| 0<=x<h.len() && h[x]==Choice{node:v,term,candidate:a.change.node};
        let y=choose|y:int| 0<=y<h.len() && h[y]==Choice{node:v,term,candidate:b.change.node};
        assert(unique(h));assert(h[x].candidate==h[y].candidate);
    }
}

}
