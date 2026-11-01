//! MAINTENANCE-ONLY deliberately wrong functions; never copied into experiment inputs.
#[path="api.rs"] mod vote_api;
use vote_api::*;
use vote_api::View;
use vstd::prelude::*;
verus! {
#[cfg(bad_deny)]
pub fn always_deny(v: View, r: Request) -> (reply: Reply)
    ensures !reply.granted,
{ Reply{node:v.config.node,term:v.state.term,candidate:r.candidate,granted:false} }
#[cfg(bad_deny)]
pub fn fixed_nonempty_witness() {
    let v=View{config:Config{node:0,members:3},state:Persistent{term:2,voted_for:None,
        log:Log{term:1,index:4}},role:Role::Follower,next:0,pending:None,halted:false};
    let r=Request{term:2,candidate:1,log:Log{term:1,index:4}};
    assert(grants(v,r));
    let actual=always_deny(v,r);
    assert(actual.granted); // fixed nonempty eligible behavior must reject this fixture
}
#[cfg(bad_vote)]
pub fn ignore_vote(v: View, r: Request) -> (reply: Reply)
    ensures reply == (Reply{node:v.config.node,term:r.term,candidate:r.candidate,granted:true}),
{ Reply{node:v.config.node,term:r.term,candidate:r.candidate,granted:true} }
#[cfg(bad_vote)]
pub fn duplicate_granted_history() {
    let v=View{config:Config{node:0,members:3},state:Persistent{term:1,voted_for:Some(1),
        log:Log{term:0,index:0}},role:Role::Follower,next:1,pending:None,halted:false};
    let a=ignore_vote(v,Request{term:1,candidate:1,log:Log{term:0,index:0}});
    let b=ignore_vote(v,Request{term:1,candidate:2,log:Log{term:0,index:0}});
    assert(unique(seq![a,b])); // actual returned outputs, not a chosen empty trace
}
#[cfg(bad_recovery)]
pub fn forget_vote(record: Record) -> (state: Persistent)
    ensures state == record.state,
{ Persistent{term:record.state.term,voted_for:None,log:record.state.log} }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn v() -> View { View{config:Config{node:0,members:3},state:Persistent{term:1,voted_for:Some(1),
        log:Log{term:0,index:0}},role:Role::Follower,next:1,pending:None,halted:false} }
    #[test] fn semantic_rejection() {
        #[cfg(bad_deny)] assert!(always_deny(v(),Request{term:1,candidate:1,log:Log{term:0,index:0}}).granted,"eligible grant required");
        #[cfg(bad_vote)] assert!(!ignore_vote(v(),Request{term:1,candidate:2,log:Log{term:0,index:0}}).granted,"same-term vote must not change");
        #[cfg(bad_recovery)] assert_eq!(forget_vote(Record{op:0,state:v().state}).voted_for,Some(1),"restored vote must not disappear");
    }
}
