//! DRAFT local scenario regression. This is not a cluster or liveness test.
#[path = "../examples/replay.rs"]
mod replay;

#[test]
fn saved_empty_duplicate_and_conflicting_append_trace_replays() {
    let trace = include_str!("../examples/reconcile.trace");
    let (passed, steps, observations) = replay::run_trace(trace).unwrap();
    assert_eq!(steps, 7);
    assert!(passed, "{observations}");
}

#[test]
fn malformed_or_empty_traces_do_not_pass() {
    assert!(replay::run_trace("seed 17\ninitial -\n").is_err());
    assert!(replay::run_trace("seed 17\ninitial -\nappend broken\n").is_err());
}
