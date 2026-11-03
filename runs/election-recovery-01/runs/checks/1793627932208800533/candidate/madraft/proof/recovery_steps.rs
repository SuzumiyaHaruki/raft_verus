//! Single-step obligations for the recovery ledger invariant.
use crate::{vote_api::*,recovery_network::*,vote_network as old};
use super::recovery_invariant::{inv,locked,disk_lock,sent_ok,returned_ok,basis,source};
use vstd::prelude::*;
verus! {
broadcast use {vstd::seq_lib::group_seq_properties,vstd::set::group_set_axioms};
pub open spec fn extension(a:Trace,b:Trace) -> bool {
    basis::messages_extend(a.net,b.net) && a.choices.is_prefix_of(b.choices)
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
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
#[verifier::rlimit(2)]
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
#[verifier::rlimit(2)]
pub proof fn ledger_step(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),unique(a.choices),
        forall|k:int| 0<=k<a.net.nodes.len() ==> locked((#[trigger] a.net.nodes[k]).state,a.choices),
        basis::local(a.net.nodes[actor(a,s)]),disk_lock(a,actor(a,s)),
        a.live[actor(a,s)] ==> basis::pending_ok(a.net,actor(a,s)),
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
pub open spec fn disk_base(t:Trace,n:int) -> bool {
    &&& basis::local(t.net.nodes[n]) && disk_lock(t,n)
    &&& (t.net.nodes[n].state.pending.is_some() ==> t.net.nodes[n].state.pending.unwrap().record.op+1==t.net.nodes[n].state.next)
    &&& (t.live[n] ==> basis::pending_ok(t.net,n))
    &&& (t.images[n].is_some() ==> valid_record(t.net.nodes[n].state.config,t.images[n].unwrap().record))
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_start(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Start,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_receive(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Receive,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_promote(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Promote,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_deliver(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Deliver,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_error(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Error,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_write(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Write,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_notify(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Notify,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_stop(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Stop,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
    assert(window(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_restart(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Restart,disk_base(a,actor(a,s)),
    ensures disk_lock(b,actor(a,s)),
        b.net.nodes[actor(a,s)].state.pending.is_some() ==> b.net.nodes[actor(a,s)].state.pending.unwrap().record.op+1==b.net.nodes[actor(a,s)].state.next,
{
    let n=actor(a,s);assert(disk_lock(a,n));
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn disk_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures forall|k:int| 0<=k<b.net.nodes.len() ==> disk_lock(b,k)
        && ((#[trigger] b.net.nodes[k]).state.pending.is_some() ==> b.net.nodes[k].state.pending.unwrap().record.op+1==b.net.nodes[k].state.next),
{
    let n=actor(a,s);assert(disk_base(a,n));
    match s {
        Step::Start(_)=>disk_start(a,b,s),
        Step::Receive(_)=>disk_receive(a,b,s),
        Step::Promote(_)=>disk_promote(a,b,s),
        Step::Deliver(_)=>disk_deliver(a,b,s),
        Step::Error(_)=>disk_error(a,b,s),
        Step::Write {..}=>disk_write(a,b,s),
        Step::Notify(_)=>disk_notify(a,b,s),
        Step::Stop(_)=>disk_stop(a,b,s),
        Step::Restart(_)=>disk_restart(a,b,s),
    }
    assert forall|k:int| 0<=k<b.net.nodes.len() implies disk_lock(b,k)
        && ((#[trigger] b.net.nodes[k]).state.pending.is_some() ==> b.net.nodes[k].state.pending.unwrap().record.op+1==b.net.nodes[k].state.next) by {
        if k!=n {assert(b.net.nodes[k]==a.net.nodes[k]);assert(b.io[k]==a.io[k]);assert(b.ready[k]==a.ready[k]);assert(b.images[k]==a.images[k]);assert(b.live[k]==a.live[k]);}
    }
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn suffix_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures forall|k:int,i:int| 0<=k<b.net.nodes.len() && 0<=i<b.net.nodes[k].choices.len() ==> b.choices.contains(#[trigger] b.net.nodes[k].choices[i]),
{
    extend(a,b,s);let n=actor(a,s);
    assert forall|k:int,i:int| 0<=k<b.net.nodes.len() && 0<=i<b.net.nodes[k].choices.len() implies b.choices.contains(#[trigger] b.net.nodes[k].choices[i]) by {
        let c=b.net.nodes[k].choices[i];
        if k!=n {assert(b.net.nodes[k]==a.net.nodes[k]);}
        if i<a.net.nodes[k].choices.len() && !(k==n && s is Restart) {
            assert(c==a.net.nodes[k].choices[i]);assert(a.choices.contains(c));lift_choice(a,b,c.node,c.candidate,c.term);
        } else {
            match s {Step::Start(_)=>{},Step::Notify(_)=>{},_=>{assert(false);}}
            assert(b.choices.last()==c);assert(b.choices.contains(c));
        }
    }
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_start(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Start,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_receive(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Receive,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_promote(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Promote,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_deliver(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Deliver,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_error(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Error,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_write(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Write,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_notify(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Notify,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_stop(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Stop,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_restart(a:Trace,b:Trace,s:Step)
    requires shape(a),enabled(a,s),b==step_after(a,s),s is Restart,disk_base(a,actor(a,s)),
    ensures b.live[actor(a,s)] ==> basis::pending_ok(b.net,actor(a,s)),
{let n=actor(a,s);}
pub proof fn pending_unchanged(a:old::Trace,b:old::Trace,n:int)
    requires basis::messages_extend(a,b),basis::pending_ok(a,n),a.nodes[n]==b.nodes[n],a.pending[n]==b.pending[n],
    ensures basis::pending_ok(b,n),
{
    if let Some(p)=a.pending[n] {if let Some(d)=p.request {assert(a.sent[d.send as int]==b.sent[d.send as int]);}}
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn pending_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures forall|k:int| 0<=k<b.net.nodes.len() && b.live[k] ==> basis::pending_ok(b.net,k),
{
    extend(a,b,s);let n=actor(a,s);assert(disk_base(a,n));
    match s {
        Step::Start(_)=>pending_start(a,b,s),
        Step::Receive(_)=>pending_receive(a,b,s),
        Step::Promote(_)=>pending_promote(a,b,s),
        Step::Deliver(_)=>pending_deliver(a,b,s),
        Step::Error(_)=>pending_error(a,b,s),
        Step::Write {..}=>pending_write(a,b,s),
        Step::Notify(_)=>pending_notify(a,b,s),
        Step::Stop(_)=>pending_stop(a,b,s),
        Step::Restart(_)=>pending_restart(a,b,s),
    }
    assert forall|k:int| 0<=k<b.net.nodes.len() && b.live[k] implies basis::pending_ok(b.net,k) by {
        if k!=n {assert(a.live[k]);assert(b.net.nodes[k]==a.net.nodes[k]);assert(b.net.pending[k]==a.net.pending[k]);assert(basis::pending_ok(a.net,k));pending_unchanged(a.net,b.net,k);}
    }
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn local_all(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures forall|k:int| 0<=k<b.net.nodes.len() ==> basis::local(#[trigger] b.net.nodes[k]),
{
    local_step(a,b,s);let n=actor(a,s);
    assert forall|k:int| 0<=k<b.net.nodes.len() implies basis::local(#[trigger] b.net.nodes[k]) by {
        if k!=n {assert(b.net.nodes[k]==a.net.nodes[k]);assert(basis::local(a.net.nodes[k]));}
    }
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn compose_nodes(b:Trace)
    requires unique(b.choices),
        forall|k:int| 0<=k<b.net.nodes.len() ==> basis::local(#[trigger] b.net.nodes[k]),
        forall|k:int| 0<=k<b.net.nodes.len() ==> locked((#[trigger] b.net.nodes[k]).state,b.choices),
        forall|k:int| 0<=k<b.net.nodes.len() ==> disk_lock(b,k) && ((#[trigger] b.net.nodes[k]).state.pending.is_some() ==> b.net.nodes[k].state.pending.unwrap().record.op+1==b.net.nodes[k].state.next),
        forall|k:int| 0<=k<b.net.nodes.len() && b.live[k] ==> basis::pending_ok(b.net,k),
        forall|k:int,i:int| 0<=k<b.net.nodes.len() && 0<=i<b.net.nodes[k].choices.len() ==> b.choices.contains(#[trigger] b.net.nodes[k].choices[i]),
    ensures forall|k:int| 0<=k<b.net.nodes.len() ==> {
        let o=#[trigger] b.net.nodes[k];
        &&& basis::local(o) && locked(o.state,b.choices) && disk_lock(b,k)
        &&& (o.state.pending.is_some() ==> o.state.pending.unwrap().record.op+1==o.state.next)
        &&& (b.live[k] ==> basis::pending_ok(b.net,k))
        &&& forall|i:int| 0<=i<o.choices.len() ==> b.choices.contains(#[trigger] o.choices[i])
    },unique(b.choices),
{ }
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn nodes_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures forall|k:int| 0<=k<b.net.nodes.len() ==> {
        let o=#[trigger] b.net.nodes[k];
        &&& basis::local(o) && locked(o.state,b.choices) && disk_lock(b,k)
        &&& (o.state.pending.is_some() ==> o.state.pending.unwrap().record.op+1==o.state.next)
        &&& (b.live[k] ==> basis::pending_ok(b.net,k))
        &&& forall|i:int| 0<=i<o.choices.len() ==> b.choices.contains(#[trigger] o.choices[i])
    },unique(b.choices),
{
    local_all(a,b,s);ledger_step(a,b,s);disk_step(a,b,s);suffix_step(a,b,s);pending_step(a,b,s);compose_nodes(b);
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn messages_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures forall|i:int| 0<=i<b.net.sent.len() ==> sent_ok(b,i),
        forall|i:int| 0<=i<b.net.replies.len() ==> returned_ok(b,i),
{
    extend(a,b,s);let n=actor(a,s);
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
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_start(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Start,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_receive(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Receive,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_promote(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Promote,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_deliver(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Deliver,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_error(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Error,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_write(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Write,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_notify(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Notify,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_stop(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Stop,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_restart(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        s is Restart,0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    extend(a,b,s);messages_step(a,b,s);let n=actor(a,s);

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn vote_case(a:Trace,b:Trace,s:Step,k:int,voter:u64)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        0<=k<b.net.nodes.len(),b.live[k],b.net.nodes[k].votes.contains(voter),
    ensures evidence(b,k,voter,b.net.nodes[k].state.state.term),
{
    match s {
        Step::Start(_)=>vote_start(a,b,s,k,voter),
        Step::Receive(_)=>vote_receive(a,b,s,k,voter),
        Step::Promote(_)=>vote_promote(a,b,s,k,voter),
        Step::Deliver(_)=>vote_deliver(a,b,s,k,voter),
        Step::Error(_)=>vote_error(a,b,s,k,voter),
        Step::Write {..}=>vote_write(a,b,s,k,voter),
        Step::Notify(_)=>vote_notify(a,b,s,k,voter),
        Step::Stop(_)=>vote_stop(a,b,s,k,voter),
        Step::Restart(_)=>vote_restart(a,b,s,k,voter),
    }
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn frame_case(a:Trace,b:Trace,s:Step,i:int)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
        0<=i<b.frames.len(),elects(b.frames[i]),
    ensures b.frames[i].was_live && promotion_allowed(b.frames[i].before.state,b.frames[i].before.votes),
        b.frames[i].after.state==promotion_after(b.frames[i].before.state,b.frames[i].before.votes),
        forall|v:u64| #[trigger] b.frames[i].before.votes.contains(v) ==> evidence(b,b.frames[i].node as int,v,b.frames[i].after.state.state.term),
{
    extend(a,b,s);let n=actor(a,s);let v=a.net.nodes[n].state;

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
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn provenance_step(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures provenance(b),forall|i:int| 0<=i<b.frames.len() ==> {
        let f=#[trigger] b.frames[i];f.node<b.net.nodes.len() && f.before.state.config==b.net.nodes[f.node as int].state.config && f.after.state.config==f.before.state.config
    },
{
    extend(a,b,s);let n=actor(a,s);assert(0<=n<usize::MAX);
    assert forall|k:int,voter:u64| 0<=k<b.net.nodes.len() && b.live[k] && #[trigger] b.net.nodes[k].votes.contains(voter)
        implies evidence(b,k,voter,b.net.nodes[k].state.state.term) by {vote_case(a,b,s,k,voter);}
    assert forall|i:int| 0<=i<b.frames.len() implies {
        let f=#[trigger] b.frames[i];f.node<b.net.nodes.len() && f.before.state.config==b.net.nodes[f.node as int].state.config && f.after.state.config==f.before.state.config
    } by {if i<a.frames.len() {assert(b.frames[i]==a.frames[i]);}else {assert(b.frames[i].node==n);}}
    assert forall|i:int| 0<=i<b.frames.len() && elects(#[trigger] b.frames[i]) implies {
        let f=b.frames[i];
        &&& f.was_live && promotion_allowed(f.before.state,f.before.votes)
        &&& f.after.state==promotion_after(f.before.state,f.before.votes)
        &&& forall|v:u64| #[trigger] f.before.votes.contains(v) ==> evidence(b,f.node as int,v,f.after.state.state.term)
    } by {frame_case(a,b,s,i);}
}
#[verifier::spinoff_prover]
#[verifier::rlimit(2)]
pub proof fn preserve(a:Trace,b:Trace,s:Step)
    requires inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures inv(b),
{
    source::preserve(a,b,s);nodes_step(a,b,s);messages_step(a,b,s);provenance_step(a,b,s);
}

}
