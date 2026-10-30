//! Executable retry/dedup guards; full replicated-state refinement is OPEN.
use vstd::prelude::*;
verus! {
pub fn next_sequence(previous: u64) -> (next: Option<u64>)
    ensures next.is_some() == (previous < u64::MAX),
        next.is_some() ==> next.unwrap() == previous + 1,
{ if previous == u64::MAX { None } else { Some(previous + 1) } }

pub fn new_request(previous: Option<u64>, received: u64) -> (yes: bool)
    ensures yes == (previous.is_none() || received > previous.unwrap()),
{
    match previous { None => true, Some(sequence) => received > sequence }
}

pub fn snapshot_boundary(decoded: u64, boundary: u64, applied: u64) -> (install: bool)
    ensures install == (decoded == boundary && boundary > applied),
{ decoded == boundary && boundary > applied }
}
