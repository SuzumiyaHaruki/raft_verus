//! Executable write planning and a crash-prefix proof for the selected FsSim.
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Stamp { pub(super) generation: u64, pub(super) term: u64, pub(super) vote: Option<u64>, pub(super) needs_snapshot: bool }
#[derive(Clone, Copy, Debug)]
pub(super) struct Plan { pub(super) first: usize, pub(super) image: Stamp }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation { CreateSnapshot, WriteSnapshot, CreateState, WriteState }

pub(super) fn plan(g0: u64, g1: u64, previous: u64, term: u64, vote: Option<u64>, needs_snapshot: bool) -> (p: Option<Plan>)
    ensures match p {
        Some(p) => p.first < 2 && p.image.generation > g0 && p.image.generation > g1
            && p.image.generation == previous + 1 && p.image.term == term && p.image.vote == vote
            && p.image.needs_snapshot == needs_snapshot
            && (p.first == 0 ==> g0 <= g1) && (p.first == 1 ==> g1 < g0),
        None => previous == u64::MAX || previous < g0 || previous < g1,
    },
{
    if previous == u64::MAX || previous < g0 || previous < g1 { return None; }
    Some(Plan { first: if g0 <= g1 { 0 } else { 1 }, image: Stamp { generation: previous + 1, term, vote, needs_snapshot } })
}
pub(super) fn operation(p: Plan, stage: usize) -> (r: (usize, Operation))
    requires p.first < 2, stage < 8,
    ensures r.0 == (if stage < 4 { p.first } else { 1 - p.first }), r.0 < 2,
        r.1 == match stage % 4 { 0 => Operation::CreateSnapshot, 1 => Operation::WriteSnapshot,
            2 => Operation::CreateState, _ => Operation::WriteState },
{
    let slot = if stage < 4 { p.first } else { 1 - p.first };
    let op = match stage % 4 { 0 => Operation::CreateSnapshot, 1 => Operation::WriteSnapshot,
        2 => Operation::CreateState, _ => Operation::WriteState };
    (slot, op)
}
pub(super) fn pair_matches(image_generation: u64, needs_snapshot: bool, snapshot_generation: Option<u64>) -> (valid: bool)
    ensures valid == (!needs_snapshot || snapshot_generation == Some(image_generation)),
{ !needs_snapshot || snapshot_generation == Some(image_generation) }
pub(super) fn prefer(generation: u64, selected: Option<u64>) -> (replace: bool)
    ensures replace == (selected.is_none() || generation > selected.unwrap()),
{
    match selected { None => true, Some(old) => generation > old }
}
pub(super) fn release_vote(image: Stamp, reply_term: u64, candidate: u64, granted: bool) -> (allowed: bool)
    ensures allowed == (!granted || (image.term == reply_term && image.vote == Some(candidate))),
{ !granted || (image.term == reply_term && image.vote == Some(candidate)) }

// A metadata image and snapshot generation are exact decoded completed writes.
// Create truncates one file; Write replaces its complete contents in FsSim.
pub struct Pair { pub metadata: Option<Stamp>, pub snapshot: Option<u64> }
pub open spec fn usable(pair: Pair) -> bool {
    pair.metadata.is_some() && (!pair.metadata.unwrap().needs_snapshot || pair.snapshot == Some(pair.metadata.unwrap().generation))
}
pub open spec fn carries(pair: Pair, image: Stamp) -> bool { usable(pair) && pair.metadata == Some(image) }
pub open spec fn effect(pair: Pair, image: Stamp, op: Operation) -> Pair {
    match op {
        Operation::CreateSnapshot => Pair { snapshot: None, ..pair },
        Operation::WriteSnapshot => Pair { snapshot: Some(image.generation), ..pair },
        Operation::CreateState => Pair { metadata: None, ..pair },
        Operation::WriteState => Pair { metadata: Some(image), ..pair },
    }
}
pub open spec fn after(first: Pair, second: Pair, image: Stamp, stage: nat) -> (Pair, Pair)
    recommends stage <= 8,
    decreases stage,
{
    if stage == 0 { (first, second) }
    else {
        let previous = after(first, second, image, (stage - 1) as nat);
        let op = match (stage - 1) % 4 { 0 => Operation::CreateSnapshot, 1 => Operation::WriteSnapshot,
            2 => Operation::CreateState, _ => Operation::WriteState };
        if stage <= 4 { (effect(previous.0, image, op), previous.1) }
        else { (previous.0, effect(previous.1, image, op)) }
    }
}
// Covers a crash after every completed create/write, including between files
// and between pairs. Recovery may use an unacknowledged newer image, but cannot
// lose the acknowledged image until a full newer pair is recoverable.
pub proof fn crash_prefix(first: Pair, second: Pair, old_image: Stamp, new_image: Stamp, stage: nat)
    requires carries(second, old_image), stage <= 8, new_image.generation > old_image.generation,
    ensures {
        let pairs = after(first, second, new_image, stage);
        carries(pairs.0, old_image) || carries(pairs.1, old_image)
            || carries(pairs.0, new_image) || carries(pairs.1, new_image)
    },
{
    if stage == 0 {} else if stage == 1 {} else if stage == 2 {} else if stage == 3 {}
    else if stage == 4 {} else if stage == 5 {} else if stage == 6 {} else if stage == 7 {} else {}
}
}
