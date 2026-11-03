//! Executable successful-write origin and retained-history connection.
use vstd::prelude::*;
#[path="../../tasks/election_recovery/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../requirements/targets.rs"] mod targets;
#[path="../../tasks/election_recovery/observation.rs"] mod election_observation;
#[path="../../tasks/election_recovery/network.rs"] mod vote_network;
#[path="../../tasks/election_recovery/recovery_network.rs"] mod recovery_network;
#[path="recovery_source.rs"] mod recovery_source;
use vote_api::*;
verus! {
pub fn finite_sources(initials:&Vec<Initial>,events:&Vec<recovery_network::Step>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let mut n=recovery_network::new_network(initials);
    proof {recovery_source::initialize(n.trace());}
    let mut k=0;
    while k<events.len()
        invariant k<=events.len(),n.represented(),n.trace().frames.len()==k,recovery_source::source_inv(n.trace()),
        decreases events.len()-k,
    {
        let e=events[k];if !recovery_network::is_enabled(&n,e) {return;}
        let ghost before=n.trace();recovery_network::execute(&mut n,e);
        proof {recovery_source::preserve(before,n.trace(),e);}
        assert(recovery_network::recovered_from_own_write(n.trace()));
        assert(n.trace().frames.drop_last()==before.frames);
        k+=1;
    }
}
}
