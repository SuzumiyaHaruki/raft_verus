//! Source induction over the fixed executor's actual Write and Restart calls.
use crate::{vote_api::*,recovery_network::*};
use vstd::prelude::*;
verus! {
pub open spec fn storage_ok(v:crate::vote_api::View) -> bool {
    valid_state(v.config,v.state) && (v.pending.is_some() ==> {
        let w=v.pending.unwrap();valid_record(v.config,w.record) && w.record.state==v.state && w.record.op<v.next
    })
}
pub open spec fn source_inv(t:Trace) -> bool {
    &&& recovered_from_own_write(t)
    &&& forall|n:int| 0<=n<t.net.nodes.len() ==> {
        let o=#[trigger] t.net.nodes[n];
        &&& storage_ok(o.state)
        &&& (t.live[n] ==> o.state.pending==match t.io[n] {Some(p)=>Some(p.proposed.write),None=>None})
        &&& (t.images[n].is_some() ==> valid_record(o.state.config,t.images[n].unwrap().record))
    }
}
pub proof fn initialize(t:Trace)
    requires boot(t),ensures source_inv(t),
{
    assert forall|n:int| 0<=n<t.net.nodes.len() implies {
        let o=#[trigger] t.net.nodes[n];
        &&& storage_ok(o.state)
        &&& (t.live[n] ==> o.state.pending==match t.io[n] {Some(p)=>Some(p.proposed.write),None=>None})
        &&& (t.images[n].is_some() ==> valid_record(o.state.config,t.images[n].unwrap().record))
    } by {assert(t.net.nodes[n].state.pending.is_none());}
}
pub proof fn source_extends(a:Trace,b:Trace,i:Image,n:int)
    requires image_source(a,i,n),b.frames.drop_last()==a.frames,
    ensures image_source(b,i,n),
{assert(b.frames[i.at as int]==a.frames[i.at as int]);}
#[verifier::spinoff_prover]
pub proof fn preserve(a:Trace,b:Trace,s:Step)
    requires source_inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),
    ensures source_inv(b),
{
    let at=actor(a,s);
    assert(0<=at<a.net.nodes.len());
    assert forall|n:int| 0<=n<b.net.nodes.len() implies {
        let o=#[trigger] b.net.nodes[n];
        &&& storage_ok(o.state)
        &&& (b.live[n] ==> o.state.pending==match b.io[n] {Some(p)=>Some(p.proposed.write),None=>None})
        &&& (b.images[n].is_some() ==> valid_record(o.state.config,b.images[n].unwrap().record))
    } by {
        if n!=at {assert(b.net.nodes[n]==a.net.nodes[n]);assert(b.io[n]==a.io[n]);assert(b.images[n]==a.images[n]);}
        else {
            match s {
                Step::Start(_)=>{},Step::Promote(_)=>{},Step::Receive(_)=>{},Step::Error(_)=>{},Step::Deliver(_)=>{},
                Step::Write{..}=>{},Step::Notify(_)=>{},Step::Stop(_)=>{},Step::Restart(_)=>{},
            }
        }
    }
    assert forall|n:int| 0<=n<b.images.len() && (#[trigger] b.images[n]).is_some()
        implies image_source(b,b.images[n].unwrap(),n) by {
        if n==at && s== (Step::Write{node:at as usize,success:true}) {
            assert(b.images[n].unwrap().at==a.frames.len());
            assert(b.frames[a.frames.len() as int].io==a.io[n]);
        } else {assert(b.images[n]==a.images[n]);source_extends(a,b,a.images[n].unwrap(),n);}
    }
    assert forall|k:int| 0<=k<b.frames.len() && (#[trigger] b.frames[k]).step is Restart implies {
        let f=b.frames[k];
        &&& !f.was_live && f.is_live && f.image.is_some() && image_source(b,f.image.unwrap(),f.node as int)
        &&& f.after.state==recovered_view(f.before.state.config,f.image.unwrap().record)
        &&& f.after.base==Some(f.image.unwrap().record) && f.after.choices==Seq::<Choice>::empty() && f.after.votes==Set::<u64>::empty()
    } by {
        if k<a.frames.len() {assert(b.frames[k]==a.frames[k]);source_extends(a,b,a.frames[k].image.unwrap(),a.frames[k].node as int);}
        else {assert(b.frames[k].step==s);assert(a.images[at].is_some());source_extends(a,b,a.images[at].unwrap(),at);}
    }
}
}
