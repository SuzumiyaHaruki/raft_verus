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
    ensures
        match result {
            Ok(()) => model::merged(log_view(old(log)@), previous as nat,
                log_view(incoming@), log_view(log@))
                && log@.take(previous as int) == old(log)@.take(previous as int),
            Err(ReconcileError::InvalidIndex) => previous > old(log).len() && log@ == old(log)@,
            Err(ReconcileError::ResourceExhausted) => previous <= old(log).len()
                && previous + incoming.len() > usize::MAX && log@ == old(log)@,
        },
{
    // Intentionally incomplete seed: independent tests and Verus must reject it.
    let _ = previous;
    let _ = incoming;
    Err(ReconcileError::InvalidIndex)
}
}
