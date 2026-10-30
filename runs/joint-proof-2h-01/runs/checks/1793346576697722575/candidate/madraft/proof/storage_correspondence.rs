//! Primitive filesystem effects are the disclosed pinned MadSim contract.
//! Async call correspondence remains an explicit outer obligation.
#[path = "vote_operation_history.rs"]
pub mod vote_operation_history;
pub use vote_operation_history::vote_operation::{protocol, state};
#[path = "../src/raft/storage.rs"]
pub mod storage;
use storage::{Phase, Action, Completion};
use vstd::prelude::*;
verus! {
// Incomplete includes the create/write crash gap; recovery must stop on it.
pub enum Disk { Missing, Incomplete, Image(protocol::VoteImage) }
// This relation describes the *actual submitted host call*, not a free success
// input. Serialization fidelity and write_all_at's single inode update are
// trusted filesystem/encoding contracts, disclosed separately from protocol.
pub open spec fn filesystem(action: Action, result: Completion, s: Disk, t: Disk) -> bool {
    match (action, result) {
        (Action::Create, Completion::Created) => t == Disk::Incomplete,
        (Action::Write(image), Completion::Written) => t == Disk::Image(image),
        (Action::Sync, Completion::Synced) => t == s,
        // Failure may leave a partial image; it cannot authorize a release.
        (_, Completion::Failed) => true,
        _ => false,
    }
}
pub open spec fn disk_invariant(phase: Phase, image: protocol::VoteImage, disk: Disk) -> bool {
    (phase == Phase::Sync || phase == Phase::Ready) ==> disk == Disk::Image(image)
}

pub proof fn storage_initial(image: protocol::VoteImage, disk: Disk)
    ensures disk_invariant(Phase::Create, image, disk),
{}

pub proof fn storage_preserves(phase: Phase, image: protocol::VoteImage,
    completion: Completion, before: Disk, after: Disk)
    requires disk_invariant(phase, image, before),
        filesystem(storage::action_for(phase, image), completion, before, after),
    ensures disk_invariant(storage::next_phase(phase, completion), image, after),
{}

pub proof fn release_has_image(phase: Phase, image: protocol::VoteImage, disk: Disk)
    requires disk_invariant(phase, image, disk), phase == Phase::Ready,
    ensures disk == Disk::Image(image),
{}
}
