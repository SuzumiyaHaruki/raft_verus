//! DRAFT unimplemented reconciliation seed; the experiment must generate the body and proof.
//! This exact executable source is erased by the Verus macro for Cargo and
//! checked with its contracts by crates/raft-proofs/reconcile.rs.
#[cfg(verus_keep_ghost)]
use crate::model;
#[cfg(verus_keep_ghost)]
use crate::representation::log_view;
use vstd::prelude::*;

pub use crate::types::{RequestId, Command, Entry, ReconcileError};

verus! {
/// `previous` is the number of existing entries preceding the incoming entries.
/// The caller must separately check the predecessor term, message term, commit
/// constraints and durable response timing. Entries with equal terms retain the
/// existing command, exactly as the current candidate model's `merged` specifies.
/// Allocation failure follows Rust Vec's abort behavior; no resource liveness is claimed.
pub fn reconcile_log(log: &mut Vec<Entry>, previous: usize, incoming: Vec<Entry>)
    -> (result: Result<(), ReconcileError>)
    // Generate the local contract with the body; the independent caller retains the full target.
    ensures true,

{
    // Intentionally incomplete seed: independent tests and Verus must reject it.
    let _ = previous;
    let _ = incoming;
    Err(ReconcileError::InvalidIndex)
}
}
