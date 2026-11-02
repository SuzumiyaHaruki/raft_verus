//! Standalone actual constructor and transition/source composition.
#[path="../../tasks/vote_collection/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../tasks/vote_collection/network.rs"] mod vote_network;
#[path="vote_sources.rs"] mod vote_sources;
use vstd::prelude::*;
use vote_api::*;
verus! {
pub fn initialize(initials:&Vec<Initial>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]),
{
    let n=vote_network::new_network(initials);
    proof { vote_sources::network_initial(n.trace()); vote_sources::network_conclusion(n.trace()); }
}
pub fn execute(n:&mut vote_network::Network,event:vote_network::Step)
    requires vote_network::shape(old(n).trace()), vote_sources::network_inv(old(n).trace()), vote_network::enabled(old(n).trace(),event),
    ensures vote_sources::network_inv(n.trace()),vote_network::sourced(n.trace()),
{
    let ghost before=n.trace();
    vote_network::execute(n,event);
    proof {
        vote_sources::network_step(before,n.trace(),event);
        vote_sources::network_conclusion(n.trace());
    }
}
}
