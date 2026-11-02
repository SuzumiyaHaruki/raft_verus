//! Actual constructors and first receiver behavior, using fixed network plumbing.
#[path="../../tasks/vote_collection/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../tasks/vote_collection/network.rs"] mod vote_network;
use vstd::prelude::*;
use vote_api::*;
verus! {
pub fn receiver(c:Config, initial:Initial, request:Request, notice:Completion) {
    if let Some(mut core)=vote_core::Core::new(c,initial) {
        let ghost before=core.view();
        let out=core.receive(request);
        assert(out==receive_result(before,request));
        let ghost pending=core.view();
        let action=core.complete(notice);
        assert(action==complete_result(pending,notice));
    }
}
pub fn initialize(initials:&Vec<Initial>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]),
{
    let n=vote_network::new_network(initials);
    assert(vote_network::boot(n.trace()));
    assert(vote_network::sourced(n.trace()));
}
}
