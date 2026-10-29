//! Same-source local verification. This entrypoint does not claim a global
//! reachable-history theorem or verification of the async host.
#[path = "../src/raft/state.rs"]
mod state;
#[path = "../src/raft/log.rs"]
mod log;
#[path = "../src/raft/quorum.rs"]
mod quorum;
#[path = "../src/raft/cursor.rs"]
mod cursor;
#[path = "../src/raft/core.rs"]
mod core;
#[path = "../src/raft/storage.rs"]
mod storage;
#[path = "../../requirements/targets.rs"]
mod targets;
mod initialization;
