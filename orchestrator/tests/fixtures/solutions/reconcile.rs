//! DRAFT: local AppendEntries suffix reconciliation, not a complete handler.
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
    if previous > log.len() {
        return Err(ReconcileError::InvalidIndex);
    }
    if incoming.len() > usize::MAX - previous {
        return Err(ReconcileError::ResourceExhausted);
    }
    let ghost original = log@;
    let mut k: usize = 0;
    while k < incoming.len()
        invariant
            log@ == original,
            original == old(log)@,
            previous <= log.len(),
            previous + k <= log.len(),
            previous + incoming.len() <= usize::MAX,
            k <= incoming.len(),
            forall |j: int| 0 <= j < k ==> previous + j < original.len()
                && original[previous as int + j].term == incoming@[j].term,
        decreases incoming.len() - k,
    {
        let index = previous + k;
        if index >= log.len() || log[index].term != incoming[k].term {
            proof {
                assert(index >= log_view(original).len()
                    || log_view(original)[index as int].term != log_view(incoming@)[k as int].term);
            }
            let ghost initial_incoming = incoming@;
            let mut remaining = incoming;
            let mut suffix = remaining.split_off(k);
            log.truncate(index);
            log.append(&mut suffix);
            proof {
                assert(log@ == original.take(index as int)
                    + initial_incoming.subrange(k as int, initial_incoming.len() as int));
                assert(log_view(log@) =~= log_view(original).take(index as int)
                    + log_view(initial_incoming).subrange(k as int, initial_incoming.len() as int));
                assert forall |j: int| 0 <= j < k implies
                    previous + j < log_view(original).len()
                    && log_view(original)[previous as int + j].term == log_view(initial_incoming)[j].term by {};
                assert(exists |w: nat| w == k && w < log_view(initial_incoming).len()
                    && (forall |j: int| 0 <= j < w ==> previous + j < log_view(original).len()
                        && log_view(original)[previous as int + j].term == log_view(initial_incoming)[j].term)
                    && (previous + w >= log_view(original).len()
                        || log_view(original)[(previous + w) as int].term != log_view(initial_incoming)[w as int].term)
                    && log_view(log@) == log_view(original).take((previous + w) as int)
                        + log_view(initial_incoming).subrange(w as int, log_view(initial_incoming).len() as int));
                assert(model::merged(log_view(original), previous as nat,
                    log_view(initial_incoming), log_view(log@)));
                assert(log@.take(previous as int) =~= original.take(previous as int));
            }
            return Ok(());
        }
        k += 1;
    }
    proof {
        assert(previous + incoming.len() <= original.len());
        assert forall |j: int| 0 <= j < incoming.len() implies
            log_view(original)[previous as int + j].term == log_view(incoming@)[j].term by {};
    }
    // MATCHING_TAIL: a wholly matching request, including empty, changes nothing.
    Ok(())
}
}
