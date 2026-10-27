//! DRAFT local proof entrypoint. This checks the library's exact executable body.
//! It does not establish complete AppendEntries, safety or liveness.
use vstd::prelude::*;
#[path = "../raft-spec/src/lib.rs"]
pub mod model;
#[path = "../raft-verified/src/types.rs"]
pub mod types;
#[path = "../raft-verified/src/reconcile.rs"]
pub mod reconcile;
pub mod reconcile_lemmas;
#[path = "../raft-spec/src/reconcile_view.rs"]
pub mod representation;

verus! {
/// Independent consumer of the reviewed local target. Ordinary implementation
/// repair cannot weaken this contract by changing the executable's annotation.
/// This delegates once to the real library operation; it is not another algorithm.
pub fn check_reconcile_contract(
    log: &mut Vec<reconcile::Entry>, previous: usize, incoming: Vec<reconcile::Entry>,
) -> (result: Result<(), reconcile::ReconcileError>)
    ensures
        match result {
            Ok(()) => model::merged(representation::log_view(old(log)@), previous as nat,
                representation::log_view(incoming@), representation::log_view(log@))
                && log@.take(previous as int) == old(log)@.take(previous as int),
            Err(reconcile::ReconcileError::InvalidIndex) =>
                previous > old(log).len() && log@ == old(log)@,
            Err(reconcile::ReconcileError::ResourceExhausted) =>
                previous <= old(log).len() && previous + incoming.len() > usize::MAX
                    && log@ == old(log)@,
        },
        result.is_ok() && incoming.len() == 0 ==>
            representation::log_view(log@) == representation::log_view(old(log)@),
{
    let result = reconcile::reconcile_log(log, previous, incoming);
    proof {
        if result.is_ok() && incoming.len() == 0 {
            reconcile_lemmas::merged_empty_preserves_log(
                representation::log_view(old(log)@), previous as nat, representation::log_view(log@));
        }
    }
    result
}
}
