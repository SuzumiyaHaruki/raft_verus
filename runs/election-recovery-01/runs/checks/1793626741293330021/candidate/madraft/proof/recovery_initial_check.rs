//! Executable successful-write origin and retained-history connection.
use vstd::prelude::*;
#[path="../../tasks/election_recovery/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../requirements/targets.rs"] mod targets;
#[path="../../tasks/election_recovery/observation.rs"] mod election_observation;
#[path="../../tasks/election_recovery/network.rs"] mod vote_network;
#[path="../../tasks/election_recovery/recovery_network.rs"] mod recovery_network;
#[path="recovery_invariant.rs"] mod recovery_invariant;
use vote_api::*;
verus! {
pub fn actual_recovery_initial(initials:&Vec<Initial>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let n=recovery_network::new_network(initials);
    proof {recovery_invariant::initialize(n.trace());recovery_invariant::conclude(n.trace());}
    assert(recovery_network::recovered_from_own_write(n.trace()));
    assert(unique(n.trace().choices) && recovery_network::provenance(n.trace()) && recovery_network::historical_safety(n.trace()));
}
}
