//! Actual migration guards; payload provenance/history correspondence is OPEN.
use vstd::prelude::*;
verus! {
pub fn next_configuration(current: u64, next: u64, complete: bool, old_gid: u64, gid: u64) -> (yes: bool)
    ensures yes == (current < u64::MAX && next == current + 1 && complete
        && gid != 0 && (old_gid == 0 || old_gid == gid)),
{
    current < u64::MAX && next == current + 1 && complete && gid != 0 && (old_gid == 0 || old_gid == gid)
}
pub fn can_import(epoch: u64, received: u64, owned: bool, ready: bool) -> (yes: bool)
    ensures yes == (epoch == received && owned && !ready),
{ epoch == received && owned && !ready }
}
