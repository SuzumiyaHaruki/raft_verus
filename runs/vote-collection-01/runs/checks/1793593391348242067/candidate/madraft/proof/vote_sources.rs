//! Provenance of actual fixed Network transitions.
use crate::{vote_api::*,vote_network::*};
use vstd::prelude::*;
verus! {
pub open spec fn pending_origin(t:Trace,a:int) -> bool {
    let o=t.nodes[a];
    &&& valid_state(o.state.config,o.state.state)
    &&& t.pending[a].is_some()==o.state.pending.is_some()
    &&& t.pending[a].is_some() ==> {
        let p=t.pending[a].unwrap();
        &&& o.state.pending==Some(p.write)
        &&& !o.state.halted
        &&& p.write.record.state==o.state.state
        &&& match p.write.action {
            Action::Send(sent) => {
                &&& p.request.is_none()
                &&& sent.config==o.state.config
                &&& sent.request==Request{term:o.state.state.term,candidate:a as u64,log:o.state.state.log}
                &&& o.state.state.voted_for==Some(a as u64)
                &&& o.state.role==Role::Candidate && o.state.round==Some(o.state.state.term)
            },
            Action::Reply(r) => {
                &&& p.request.is_some()
                &&& routed(t,p.request.unwrap())
                &&& p.request.unwrap().peer==a
                &&& r.node==a
                &&& r.candidate==t.sent[p.request.unwrap().send as int].payload.request.candidate
                &&& r.granted ==> r.term==t.sent[p.request.unwrap().send as int].payload.request.term
            },
            Action::Persisted => p.request.is_none(),
        }
    }
}
pub open spec fn sent_origin(t:Trace,i:int) -> bool {
    let s=t.sent[i];
    &&& s.node<t.nodes.len()
    &&& s.payload.config==t.nodes[s.node as int].state.config
    &&& s.payload.request.candidate==s.node
    &&& valid_request(s.payload.config,s.payload.request)
    &&& s.notice.success && s.notice.op==s.write.record.op
    &&& s.write.record.state.term==s.payload.request.term
    &&& s.write.record.state.voted_for==Some(s.node)
}
pub open spec fn reply_origin(t:Trace,i:int) -> bool {
    let r=t.replies[i];
    &&& r.request.is_some()
    &&& routed(t,r.request.unwrap())
    &&& r.request.unwrap().peer==r.node && r.payload.node==r.node
    &&& r.payload.candidate==t.sent[r.request.unwrap().send as int].payload.request.candidate
    &&& r.payload.granted ==> r.payload.term==t.sent[r.request.unwrap().send as int].payload.request.term
}
pub closed spec fn network_inv(t: Trace) -> bool {
    &&& forall|a:int| 0<=a<t.nodes.len() ==> pending_origin(t,a)
    &&& forall|i:int| 0<=i<t.sent.len() ==> sent_origin(t,i)
    &&& forall|i:int| 0<=i<t.replies.len() ==> reply_origin(t,i)
    &&& forall|a:int,v:u64| 0<=a<t.nodes.len() && #[trigger] t.nodes[a].votes.contains(v)
        ==> if v==a { self_evidence(t,a,t.nodes[a].state.state.term) }
            else { remote_evidence(t,a,t.nodes[a].state.state.term,v) }
}
pub proof fn network_initial(t: Trace)
    requires boot(t),
    ensures network_inv(t),
{ }
pub open spec fn evidence(t:Trace,a:int,term:u64,v:u64) -> bool {
    if v==a { self_evidence(t,a,term) } else { remote_evidence(t,a,term,v) }
}
pub proof fn retain_evidence(b:Trace,t:Trace,a:int,term:u64,v:u64)
    requires evidence(b,a,term,v), t.nodes[a].state.config==b.nodes[a].state.config,
        b.sent.len()<=t.sent.len(),b.replies.len()<=t.replies.len(),b.delivered.len()<=t.delivered.len(),
        forall|i:int| 0<=i<b.sent.len() ==> #[trigger] t.sent[i]==b.sent[i],
        forall|i:int| 0<=i<b.replies.len() ==> #[trigger] t.replies[i]==b.replies[i],
        forall|i:int| 0<=i<b.delivered.len() ==> #[trigger] t.delivered[i]==b.delivered[i],
    ensures evidence(t,a,term,v),
{
    if v==a {
        let i=choose|i:int| 0<=i<b.sent.len() && (#[trigger] b.sent[i]).node==a
            && b.sent[i].notice.success && b.sent[i].notice.op==b.sent[i].write.record.op
            && b.sent[i].payload.config==b.nodes[a].state.config
            && b.sent[i].payload.request.term==term && b.sent[i].payload.request.candidate==a
            && b.sent[i].write.record.state.term==term && b.sent[i].write.record.state.voted_for==Some(a as u64);
        assert(t.sent[i]==b.sent[i]);
        assert(self_evidence(t,a,term));
    } else {
        let j=choose|j:int| 0<=j<b.delivered.len() && {
            let i=#[trigger] b.delivered[j];
            &&& i<b.replies.len() && b.replies[i as int].request.is_some()
            &&& { let r=b.replies[i as int];let d=r.request.unwrap();
                &&& d.send<b.sent.len() && b.sent[d.send as int].node==a
                &&& b.sent[d.send as int].payload.request.candidate==a && b.sent[d.send as int].payload.request.term==term
                &&& d.peer==v && r.node==v && r.payload.node==v
                &&& r.payload.candidate==a && r.payload.term==term && r.payload.granted }
        };
        let i=b.delivered[j];let d=b.replies[i as int].request.unwrap();
        assert(t.delivered[j]==i && t.replies[i as int]==b.replies[i as int] && t.sent[d.send as int]==b.sent[d.send as int]);
        assert(remote_evidence(t,a,term,v));
    }
}
pub proof fn network_step(before: Trace, after: Trace, event: Step)
    requires network_inv(before), shape(before), enabled(before,event), after == step_after(before,event),
    ensures network_inv(after),
{
    let at=match event {
        Step::Start(a) | Step::Save{node:a,success:_} => a as int,
        Step::Receive(d) => d.peer as int,
        Step::Error(d) => before.sent[d.send as int].node as int,
        Step::Deliver(i) => before.sent[before.replies[i as int].request.unwrap().send as int].node as int,
    };
    assert(0<=at<before.nodes.len());
    assert(pending_origin(before,at));
    assert(after.nodes.len()==before.nodes.len());
    assert forall|a:int| 0<=a<after.nodes.len() implies (#[trigger] after.nodes[a]).state.config==before.nodes[a].state.config by {
        if a!=at { assert(after.nodes[a]==before.nodes[a]); }
        else { match event { Step::Start(_) => {},Step::Receive(_) => {},Step::Save{..} => {},Step::Deliver(_) => {},Step::Error(_) => {} } }
    }
    assert forall|i:int| 0<=i<before.sent.len() implies #[trigger] after.sent[i]==before.sent[i] by { match event { Step::Save{..} => {}, _ => {} } }
    assert forall|i:int| 0<=i<before.replies.len() implies #[trigger] after.replies[i]==before.replies[i] by { match event { Step::Save{..} => {}, _ => {} } }
    assert forall|i:int| 0<=i<before.delivered.len() implies #[trigger] after.delivered[i]==before.delivered[i] by { match event { Step::Deliver(_) => {}, _ => {} } }
    assert(before.sent.len()<=after.sent.len() && before.replies.len()<=after.replies.len() && before.delivered.len()<=after.delivered.len());
    assert forall|a:int| 0<=a<after.nodes.len() implies pending_origin(after,a) by {
        assert(pending_origin(before,a));
        if a==at {
            match event {
                Step::Start(_) => {},
                Step::Receive(d) => { assert(sent_origin(before,d.send as int)); },
                Step::Save{..} => {},
                Step::Deliver(i) => { assert(reply_origin(before,i as int)); },
                Step::Error(d) => { assert(sent_origin(before,d.send as int)); },
            }
        }
    }
    assert forall|i:int| 0<=i<after.sent.len() implies sent_origin(after,i) by {
        if i<before.sent.len() { assert(sent_origin(before,i)); }
        else { match event { Step::Save{node,success} => {}, _ => {} } }
    }
    assert forall|i:int| 0<=i<after.replies.len() implies reply_origin(after,i) by {
        if i<before.replies.len() { assert(reply_origin(before,i)); }
        else { match event { Step::Save{node,success} => {}, _ => {} } }
    }
    assert forall|a:int,v:u64| 0<=a<after.nodes.len() && #[trigger] after.nodes[a].votes.contains(v)
        implies if v==a { self_evidence(after,a,after.nodes[a].state.state.term) }
            else { remote_evidence(after,a,after.nodes[a].state.state.term,v) } by {
        let term=after.nodes[a].state.state.term;
        let new_self=match event {
            Step::Save{node,success} => node==a && success && before.pending[a].unwrap().write.action is Send,
            _ => false,
        };
        let new_remote=match event {
            Step::Deliver(i) => { let r=before.replies[i as int];let d=r.request.unwrap();
                a==at && v==d.peer && collect_result(before.nodes[a].state,before.nodes[a].votes,context(before,d),reply_result(r))==Collect::Counted },
            _ => false,
        };
        if new_self {
            assert(v==a);
            assert(sent_origin(after,before.sent.len() as int));
            assert(self_evidence(after,a,term));
        } else if new_remote {
            let i=match event { Step::Deliver(i)=>i, _=>0 };
            let r=before.replies[i as int]; let d=r.request.unwrap();
            assert(reply_origin(before,i as int));
            assert(sent_origin(before,d.send as int));
            assert(after.delivered[before.delivered.len() as int]==i);
            assert(remote_evidence(after,a,term,v));
        } else {
            match event {
                Step::Start(_) => {}, Step::Receive(_) => {}, Step::Error(_) => {},
                Step::Save{..} => {}, Step::Deliver(_) => {},
            }
            assert(before.nodes[a].votes.contains(v));
            assert(term==before.nodes[a].state.state.term);
            assert(evidence(before,a,term,v));
            retain_evidence(before,after,a,term,v);
        }
    }

}
pub proof fn network_conclusion(t: Trace)
    requires network_inv(t), shape(t),
    ensures sourced(t),
{ }
}
