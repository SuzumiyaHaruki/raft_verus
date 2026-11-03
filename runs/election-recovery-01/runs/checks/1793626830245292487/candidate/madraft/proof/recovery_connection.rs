//! Executable successful-write origin and retained-history connection.
use vstd::prelude::*;
#[path="../../tasks/election_recovery/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="../../requirements/targets.rs"] mod targets;
#[path="../../tasks/election_recovery/observation.rs"] mod election_observation;
#[path="../../tasks/election_recovery/network.rs"] mod vote_network;
#[path="../../tasks/election_recovery/recovery_network.rs"] mod recovery_network;
use vote_api::*;
verus! {
#[verifier::spinoff_prover]
pub fn actual_written_restart(initials:&Vec<Initial>,node:usize,notify:bool)
    requires 0<initials.len()<=u64::MAX,node<initials.len(),
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    use recovery_network::{Step as S,execute};
    let mut n=recovery_network::new_network(initials);
    execute(&mut n,S::Start(node));
    if n.io[node].is_none() {return;}
    let actual=n.io[node].unwrap().proposed.write;
    let ghost ledger=n.trace().choices;
    assert(ledger.len()==1);
    assert(ledger[0]==Choice{node:node as u64,term:actual.record.state.term,candidate:node as u64});
    execute(&mut n,S::Write{node,success:true});
    let ghost image=n.trace().images[node as int].unwrap();
    assert(image.record==actual.record && image.epoch==0 && image.at==1);
    assert(recovery_network::image_source(n.trace(),image,node as int));
    if notify {execute(&mut n,S::Notify(node));}
    execute(&mut n,S::Stop(node));
    execute(&mut n,S::Restart(node));
    assert(n.trace().choices==ledger);
    assert(n.trace().net.nodes[node as int].state==recovered_view(Config{node:node as u64,members:initials.len() as u64},actual.record));
    assert(n.trace().net.nodes[node as int].base==Some(actual.record));
    assert(n.trace().net.nodes[node as int].choices==Seq::<Choice>::empty());
    assert(n.trace().net.nodes[node as int].votes==Set::<u64>::empty());
    assert(recovery_network::image_source(n.trace(),image,node as int));
}
}
