//! Deliberately wrong maintenance fragments, never experiment input or a solution.
#[path="api.rs"] mod vote_api;
#[cfg(any(bad_source,source_control))]
#[path="core.rs"] mod vote_core;
#[cfg(any(bad_source,source_control))]
#[path="network.rs"] mod vote_network;
use vote_api::*;
use vstd::prelude::*;
verus! {
#[cfg(bad_duplicate)]
pub fn duplicate_count() -> (out:u64)
    ensures out as nat==Set::<u64>::empty().insert(0).insert(1).len(),
{ 3 }
#[cfg(bad_context)]
pub fn replace_original(c:RpcContext,now:u64) -> (out:RpcContext)
    requires now!=c.sent.request.term,
    ensures out==c,
{ RpcContext{sent:Send{request:Request{term:now,..c.sent.request},..c.sent},..c} }
#[cfg(any(bad_source,source_control))]
pub open spec fn empty_history() -> vote_network::Trace {
    vote_network::Trace{nodes:seq![vote_network::Observation{
        state:fresh_view(Config{node:0,members:1},Initial{state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower}),
        votes:Set::empty(),choices:Seq::empty(),base:None}],pending:seq![None],sent:Seq::empty(),replies:Seq::empty(),delivered:Seq::empty()}
}
#[cfg(source_control)]
pub proof fn no_output_no_delivery()
    ensures !vote_network::enabled(empty_history(),vote_network::Step::Deliver(0)),
{ }
#[cfg(bad_source)]
pub proof fn inject_without_remote_output() {
    assert(vote_network::enabled(empty_history(),vote_network::Step::Deliver(0)));
}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn calibration() {
        #[cfg(bad_duplicate)] assert_eq!(duplicate_count(),2,"duplicate actual count must be rejected");
        #[cfg(bad_context)] {
            let original=RpcContext{sent:Send{config:Config{node:0,members:3},request:Request{term:1,candidate:0,log:Log{term:0,index:0}}},peer:1};
            assert_eq!(replace_original(original,2),original,"old RPC must not be relabeled as current");
        }
        #[cfg(bad_source)] assert!(vote_network::has_reply(&Vec::new(),0),"unbacked reply delivery is forbidden");
        #[cfg(source_control)] {
            assert!(!vote_network::has_reply(&Vec::new(),0));
            // A real but WRONG output is still a recorded output. The gate does not certify it.
            let wrong=vote_network::Returned{node:1,request:Some(vote_network::Dispatch{send:0,peer:1}),
                payload:Reply{node:99,term:0,candidate:99,granted:true}};
            assert!(vote_network::has_reply(&vec![wrong],0));
        }
    }
}
