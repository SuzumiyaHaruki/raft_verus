//! Verified components only; no aggregate Raft safety claim.
#[path = "../src/raft/election.rs"]
mod election;
#[path = "../../requirements/support.rs"]
mod support;
#[path = "segments.rs"]
mod segments;
#[path = "../src/raft/log.rs"]
mod log_core;

#[path = "../src/raft/quorum.rs"]
mod quorum;

#[path = "../src/raft/vote_history.rs"]
mod vote_history;
