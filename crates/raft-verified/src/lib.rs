//! DRAFT experiment scaffold: data types, interfaces and intentionally incomplete bodies.
//! No complete Raft node or `step` implementation is provided.
//! `u64` exhaustion must return an error; the unbounded model is not a resource proof.
#![forbid(unsafe_code)]

pub type NodeId = u64;
pub type Term = u64;
pub type LogIndex = u64;

pub mod types;
pub mod reconcile;
pub use reconcile::{reconcile_log, Command, Entry, ReconcileError, RequestId};

pub mod state;
pub mod node_state;
pub mod node;
pub use state::{DurableState, InitError, Node, Observation, Role};

pub mod io;
pub mod persistence;
pub use io::*;
