//! Necessary parent: checked operation release + concrete host filesystem effects.
//! This conditional result does not discharge actual async dispatch adequacy.
#[path = "storage_correspondence.rs"]
pub mod storage_correspondence;
use storage_correspondence::{protocol as p, state, storage as io, Disk};
use vstd::prelude::*;
verus! {
pub proof fn durable_reply(before: p::OperationView, after: p::OperationView,
    id: u64, saved: p::VoteImage, disk: Disk, reply: state::VoteReply)
    requires p::view_invariant(before),
        storage_correspondence::disk_invariant(io::Phase::Ready, saved, disk),
        p::complete_step(before, id, saved, after, Some(reply)),
    ensures disk == Disk::Image(p::image(after.state)),
        state::vote_step(before.pending.unwrap().4, before.pending.unwrap().2, after.state, reply),
        reply.granted ==> saved.term == before.pending.unwrap().2.term
            && saved.voted_for == Some(before.pending.unwrap().2.candidate),
{
    storage_correspondence::release_has_image(io::Phase::Ready, saved, disk);
    storage_correspondence::vote_operation_history::released_reply(before, id, saved, after, reply);
}

// Every intermediate storage boundary is represented, including the incomplete
// create/write gap. A crash can observe any boundary; no sync atomicity is added.
pub struct StorageHistory {
    pub image: p::VoteImage,
    pub phases: Seq<io::Phase>,
    pub disks: Seq<Disk>,
    pub completions: Seq<io::Completion>,
}
pub open spec fn storage_execution(h: StorageHistory) -> bool {
    h.phases.len() == h.completions.len() + 1 && h.disks.len() == h.phases.len()
    && h.phases[0] == io::Phase::Create
    && forall|k: int| 0 <= k < h.completions.len() ==>
        h.phases[k+1] == io::next_phase(h.phases[k], #[trigger] h.completions[k])
        && storage_correspondence::filesystem(io::action_for(h.phases[k], h.image),
            h.completions[k], h.disks[k], h.disks[k+1])
}
pub proof fn disk_at(h: StorageHistory, k: int)
    requires storage_execution(h), 0 <= k < h.phases.len(),
    ensures storage_correspondence::disk_invariant(h.phases[k], h.image, h.disks[k]),
    decreases k,
{
    if k == 0 { storage_correspondence::storage_initial(h.image, h.disks[0]); }
    else {
        disk_at(h, k-1);
        assert(h.phases[k] == io::next_phase(h.phases[k-1], h.completions[k-1]));
        storage_correspondence::storage_preserves(h.phases[k-1], h.image, h.completions[k-1],
            h.disks[k-1], h.disks[k]);
    }
}

// The actual recover constructor establishes image equality and the invariant.
// A subsequent admitted begin step therefore cannot change that saved vote.
pub proof fn restored_vote_conflict(saved: p::VoteImage, recovered: p::OperationView,
    request: state::VoteRequest, after: p::OperationView, submitted: Option<(u64, p::VoteImage)>)
    requires p::view_invariant(recovered), p::image(recovered.state) == saved,
        saved.voted_for.is_some(), request.term == saved.term,
        Some(request.candidate) != saved.voted_for,
        p::begin_step(recovered, request, after, submitted), submitted.is_some(),
    ensures after.pending.is_some(), !after.pending.unwrap().3.granted,
        after.state.voted_for == saved.voted_for,
{}

}
