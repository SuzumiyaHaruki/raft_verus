// DRAFT negative fixture; MUST be rejected and stays outside every positive profile.
use vstd::prelude::*;
verus! {
pub fn wrong_increment(n: u64) -> (result: u64)
    requires n < u64::MAX,
    ensures result == n,
{
    n + 1
}
}
