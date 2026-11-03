//! Cross-instance ledger locks over actual retained writes and recovery calls.
use crate::{vote_api::*,recovery_network::*,vote_network as old};
use vstd::prelude::*;
#[path="network_invariant.rs"] mod basis;
#[path="recovery_source.rs"] mod source;
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
    requires boot(t),ensures inv(t),
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
pub open spec fn extension(a:Trace,b:Trace) -> bool {
    basis::messages_extend(a.net,b.net) && a.choices.is_prefix_of(b.choices)
}
#[verifier::spinoff_prover]
pub proof fn extend(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),
    ensures extension(a,b),
{
    assert forall|n:int| 0<=n<a.net.nodes.len() implies (#[trigger] a.net.nodes[n]).state.config==b.net.nodes[n].state.config by {
        if n!=actor(a,s) {assert(b.net.nodes[n]==a.net.nodes[n]);}
    }
}
pub proof fn lift_choice(a:Trace,b:Trace,v:u64,n:u64,term:u64)
    requires extension(a,b),choice_for(a,v,n,term),ensures choice_for(b,v,n,term),
{
    let c=Choice{node:v,term,candidate:n};
    let i=choose|i:int| 0<=i<a.choices.len() && a.choices[i]==c;
    assert(b.choices[i]==c);
}
pub proof fn lift_evidence(a:Trace,b:Trace,n:int,v:u64,term:u64)
    requires extension(a,b),0<=n<a.net.nodes.len(),evidence(a,n,v,term),
    ensures evidence(b,n,v,term),
{lift_choice(a,b,v,n as u64,term);basis::lift_evidence(a.net,b.net,n,v,term);}
#[verifier::spinoff_prover]
pub proof fn local_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures basis::local(b.net.nodes[actor(a,s)]),
{
    let n=actor(a,s);
    match s {
        Step::Restart(_)=>{},Step::Stop(_)=>{},Step::Write{..}=>{},
        _=>{
            let e=call_step(a,s);let q=old::step_after(a.net,e);
            assert(old::enabled(a.net,e));
            basis::actor_local(a.net,q,e);
            assert(q.nodes[n]==b.net.nodes[n]);
        }
    }
}
pub proof fn lock_advance(v:crate::vote_api::View,u:crate::vote_api::View,h:Seq<Choice>)
    requires locked(v,h),u.config==v.config,u.state.term>=v.state.term,
        u.state.term==v.state.term && v.state.voted_for.is_some() ==> u.state.voted_for==v.state.voted_for,
    ensures locked(u,h),
{
    assert forall|i:int| 0<=i<h.len() && (#[trigger] h[i]).node==u.config.node implies h[i].term<=u.state.term
        && (h[i].term==u.state.term ==> u.state.voted_for==Some(h[i].candidate)) by {assert(h[i].term<=v.state.term);}
}
pub proof fn append_locked(v:crate::vote_api::View,h:Seq<Choice>,c:Choice)
    requires unique(h),locked(v,h),c.node==v.config.node,c.term==v.state.term,v.state.voted_for==Some(c.candidate),
    ensures unique(h.push(c)),locked(v,h.push(c)),
{
    let next=h.push(c);
    assert forall|i:int,j:int| 0<=i<next.len() && 0<=j<next.len()
        && (#[trigger] next[i]).node==(#[trigger] next[j]).node && next[i].term==next[j].term
        implies next[i].candidate==next[j].candidate by {
        if i<h.len() && j<h.len() {assert(next[i]==h[i]);assert(next[j]==h[j]);}
        else if i<h.len() {assert(h[i].node==v.config.node && h[i].term==v.state.term);}
        else if j<h.len() {assert(h[j].node==v.config.node && h[j].term==v.state.term);}
    }
    assert forall|i:int| 0<=i<next.len() && (#[trigger] next[i]).node==v.config.node implies next[i].term<=v.state.term
        && (next[i].term==v.state.term ==> v.state.voted_for==Some(next[i].candidate)) by {if i<h.len() {assert(next[i]==h[i]);}}
}
#[verifier::spinoff_prover]
pub proof fn ledger_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures unique(b.choices),forall|k:int| 0<=k<b.net.nodes.len() ==> locked((#[trigger] b.net.nodes[k]).state,b.choices),
{
    let n=actor(a,s);let v=a.net.nodes[n].state;let u=b.net.nodes[n].state;
    assert(locked(v,a.choices));
    if s is Restart {assert(!a.live[n]);assert(disk_lock(a,n));assert(u.state==v.state);}
    lock_advance(v,u,a.choices);
    if b.choices!=a.choices {
        let c=b.choices.last();
        assert(b.choices==a.choices.push(c));
        assert(c.node==u.config.node && c.term==u.state.term && u.state.voted_for==Some(c.candidate));
        append_locked(u,a.choices,c);
    }
    assert forall|k:int| 0<=k<b.net.nodes.len() implies locked((#[trigger] b.net.nodes[k]).state,b.choices) by {
        if k!=n {assert(a.net.nodes[k]==b.net.nodes[k]);}
        assert forall|i:int| 0<=i<b.choices.len() && (#[trigger] b.choices[i]).node==b.net.nodes[k].state.config.node implies b.choices[i].term<=b.net.nodes[k].state.state.term
            && (b.choices[i].term==b.net.nodes[k].state.state.term ==> b.net.nodes[k].state.state.voted_for==Some(b.choices[i].candidate)) by {
            if i<a.choices.len() {assert(b.choices[i]==a.choices[i]);}
        }
    }
}
#[verifier::spinoff_prover]
pub proof fn disk_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures forall|k:int| 0<=k<b.net.nodes.len() ==> disk_lock(b,k)
        && ((#[trigger] b.net.nodes[k]).state.pending.is_some() ==> b.net.nodes[k].state.pending.unwrap().record.op+1==b.net.nodes[k].state.next),
{
    let n=actor(a,s);
    assert forall|k:int| 0<=k<b.net.nodes.len() implies disk_lock(b,k)
        && ((#[trigger] b.net.nodes[k]).state.pending.is_some() ==> b.net.nodes[k].state.pending.unwrap().record.op+1==b.net.nodes[k].state.next) by {
        if k!=n {assert(b.net.nodes[k]==a.net.nodes[k]);assert(b.io[k]==a.io[k]);assert(b.ready[k]==a.ready[k]);assert(b.images[k]==a.images[k]);assert(b.live[k]==a.live[k]);}
        else {assert(disk_lock(a,k));match s {Step::Stop(_)=>{assert(window(a,k));},Step::Notify(_)=>{},Step::Write{..}=>{},_=>{}}}
    }
}
#[verifier::spinoff_prover]
pub proof fn preserve(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures inv(b),
{
    source::preserve(a,b,s);
    extend(a,b,s);local_step(a,b,s);ledger_step(a,b,s);disk_step(a,b,s);
    let n=actor(a,s);let v=a.net.nodes[n].state;let u=b.net.nodes[n].state;
    assert(0<=n<a.net.nodes.len());
    assert forall|k:int| 0<=k<b.net.nodes.len() implies {
        let o=#[trigger] b.net.nodes[k];
        &&& basis::local(o) && locked(o.state,b.choices) && disk_lock(b,k)
        &&& (o.state.pending.is_some() ==> o.state.pending.unwrap().record.op+1==o.state.next)
        &&& (b.live[k] ==> basis::pending_ok(b.net,k))
        &&& forall|i:int| 0<=i<o.choices.len() ==> b.choices.contains(#[trigger] o.choices[i])
    } by {
        if k!=n {assert(b.net.nodes[k]==a.net.nodes[k]);}
        else {match s {Step::Restart(_)=>{},Step::Stop(_)=>{},Step::Write{..}=>{},_=>{}}}
    }
    assert(unique(b.choices));
    assert forall|i:int| 0<=i<b.net.sent.len() implies sent_ok(b,i) by {
        if i<a.net.sent.len() {assert(sent_ok(a,i));assert(b.net.sent[i]==a.net.sent[i]);let sent=a.net.sent[i];lift_choice(a,b,sent.node,sent.node,sent.payload.request.term);}
        else {
            assert(s is Notify);let sent=b.net.sent[i];let p=a.io[n].unwrap();
            assert(a.net.nodes[n].state.pending==Some(p.proposed.write));
            assert(p.proposed.write.action==Action::Send(sent.payload));
            assert(a.net.nodes[n].choices.contains(Choice{node:sent.node,term:sent.payload.request.term,candidate:sent.node}));
            assert(choice_for(a,sent.node,sent.node,sent.payload.request.term));lift_choice(a,b,sent.node,sent.node,sent.payload.request.term);
        }
    }
    assert forall|i:int| 0<=i<b.net.replies.len() implies returned_ok(b,i) by {
        if i<a.net.replies.len() {assert(returned_ok(a,i));assert(b.net.replies[i]==a.net.replies[i]);let r=a.net.replies[i];if r.payload.granted {lift_choice(a,b,r.node,r.payload.candidate,r.payload.term);}}
        else {
            assert(s is Notify);let r=b.net.replies[i];let p=a.io[n].unwrap();
            assert(a.net.nodes[n].state.pending==Some(p.proposed.write));
            assert(p.proposed.write.action==Action::Reply(r.payload));
            if r.payload.granted {assert(b.choices.last()==Choice{node:r.node,term:r.payload.term,candidate:r.payload.candidate});assert(choice_for(b,r.node,r.payload.candidate,r.payload.term));}
        }
    }
    assert forall|k:int,voter:u64| 0<=k<b.net.nodes.len() && b.live[k] && #[trigger] b.net.nodes[k].votes.contains(voter)
        implies evidence(b,k,voter,b.net.nodes[k].state.state.term) by {
        let term=b.net.nodes[k].state.state.term;
        if a.live[k] && a.net.nodes[k].votes.contains(voter) && a.net.nodes[k].state.state.term==term {
            assert(evidence(a,k,voter,term));lift_evidence(a,b,k,voter,term);
        } else {
            match s {
                Step::Notify(_)=>{
                    assert(k==n && voter==k);let i=a.net.sent.len() as int;
                    assert(sent_ok(b,i));assert(b.net.sent[i].node==k && b.net.sent[i].payload.request.term==term);
                    assert(old::self_evidence(b.net,k,term));
                },
                Step::Deliver(i)=>{
                    let r=a.net.replies[i as int];let d=r.request.unwrap();
                    assert(k==n && voter==d.peer);assert(returned_ok(a,i as int));assert(sent_ok(a,d.send as int));
                    assert(b.net.delivered[a.net.delivered.len() as int]==i);
                    assert(old::remote_evidence(b.net,k,term,voter));
                    lift_choice(a,b,voter,k as u64,term);
                },_=>{assert(false);},
            }
        }
    }
    assert forall|i:int| 0<=i<b.frames.len() implies {
        let f=#[trigger] b.frames[i];f.node<b.net.nodes.len() && f.before.state.config==b.net.nodes[f.node as int].state.config
            && f.after.state.config==f.before.state.config
    } by {if i<a.frames.len() {assert(b.frames[i]==a.frames[i]);}else {assert(b.frames[i].node==n);}}
    assert forall|i:int| 0<=i<b.frames.len() && elects(#[trigger] b.frames[i]) implies {
        let f=b.frames[i];
        &&& f.was_live && promotion_allowed(f.before.state,f.before.votes)
        &&& f.after.state==promotion_after(f.before.state,f.before.votes)
        &&& forall|v:u64| #[trigger] f.before.votes.contains(v) ==> evidence(b,f.node as int,v,f.after.state.state.term)
    } by {
        let f=b.frames[i];
        if i<a.frames.len() {
            assert(f==a.frames[i]);
            assert forall|v:u64| #[trigger] f.before.votes.contains(v) implies evidence(b,f.node as int,v,f.after.state.state.term) by {
                assert(evidence(a,f.node as int,v,f.after.state.state.term));lift_evidence(a,b,f.node as int,v,f.after.state.state.term);
            }
        } else {
            match s {Step::Promote(_)=>{},_=>{assert(false);}}
            assert(f.was_live && promotion_allowed(v,f.before.votes));
            assert forall|voter:u64| #[trigger] f.before.votes.contains(voter) implies evidence(b,f.node as int,voter,f.after.state.state.term) by {
                assert(evidence(a,n,voter,v.state.term));lift_evidence(a,b,n,voter,v.state.term);
            }
        }
    }

}
#[verifier::spinoff_prover]
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
