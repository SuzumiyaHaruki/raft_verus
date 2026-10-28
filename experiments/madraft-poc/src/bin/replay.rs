//! DRAFT fixed-seed actual MadSim replay, with normalized event/effect history.
use raft_host_madsim::cluster::{run, Scenario};
fn main() {
    let seed = 771;
    let a = run(Scenario::Duplicate, seed);
    let b = run(Scenario::Duplicate, seed);
    assert_eq!(a, b, "seed did not reproduce the actual execution");
    for record in &a.records {
        println!("{record:?}");
    }
    println!(
        "CLUSTER_REPLAY_PASS seed={seed} steps={} nodes={} elections={} applications={}",
        a.records.len(),
        a.nodes.len(),
        a.elections.len(),
        a.applications.len()
    );
}
