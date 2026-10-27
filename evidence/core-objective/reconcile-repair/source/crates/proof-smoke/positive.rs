// DRAFT: standalone Verus smoke, not a Raft theorem.
use vstd::prelude::*;
verus! {
pub open spec fn successor(n: nat) -> nat { n + 1 }

pub proof fn successor_is_larger(n: nat)
    ensures successor(n) > n,
{}

pub fn increment(n: u64) -> (result: u64)
    requires n < u64::MAX,
    ensures result as nat == successor(n as nat), result > n,
{
    proof { successor_is_larger(n as nat); }
    n + 1
}
}
