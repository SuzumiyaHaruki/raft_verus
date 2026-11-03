#[path="../../tasks/election_recovery/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../requirements/targets.rs"] mod targets;
#[path="../../tasks/election_recovery/observation.rs"] mod election_observation;
#[path="../../tasks/election_recovery/network.rs"] mod vote_network;
#[path="network_invariant.rs"] mod network_invariant;
use vstd::prelude::*;
use vote_api::*;
verus! {
pub fn finite_actual_network(initials:&Vec<Initial>,events:&Vec<vote_network::Step>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let mut n=vote_network::new_network(initials);
    proof {network_invariant::initialize(n.trace());network_invariant::conclude(n.trace());}
    let mut k=0;
    while k<events.len()
        invariant k<=events.len(),vote_network::shape(n.trace()),network_invariant::inv(n.trace()),
        decreases events.len()-k,
    {
        let e=events[k];if !vote_network::is_enabled(&n,e) {return;}
        let ghost before=n.trace();vote_network::execute(&mut n,e);
        proof {network_invariant::preserve(before,n.trace(),e);network_invariant::conclude(n.trace());}
        assert(vote_network::sourced(n.trace()) && vote_network::choice_sourced(n.trace()));
        assert(vote_network::elections_justified(n.trace()) && vote_network::historical_safety(n.trace()));
        assert(n.trace().frames.drop_last()==before.frames);k+=1;
    }
}
}
