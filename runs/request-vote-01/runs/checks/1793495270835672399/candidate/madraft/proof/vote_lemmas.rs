//! Finite-output uniqueness and faithful-record restart composition.
use crate::{vote_api::*, vote_core::{Core,bounded,rooted}};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(tracked c: &Core) ensures unique(c.replies()) { c.facts(); }
#[cfg(any(rv_confirmed, rv_pending))]
pub proof fn restart_composition(tracked before: &Core, tracked after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.replies() + after.replies()),
{
    before.facts(); after.facts();
    let a = before.replies(); let b = after.replies(); let h = a + b;
    assert forall|i:int,j:int| 0 <= i < h.len() && 0 <= j < h.len()
        && (#[trigger] h[i]).granted && (#[trigger] h[j]).granted
        && h[i].node == h[j].node && h[i].term == h[j].term
        implies h[i].candidate == h[j].candidate by {
        if i < a.len() && j < a.len() { assert(a[i].candidate == a[j].candidate); }
        else if i >= a.len() && j >= a.len() {
            assert(b[i-a.len()].candidate == b[j-a.len()].candidate);
        } else if i < a.len() {
            assert(bounded(a[i],record.state));
            assert(rooted(b[j-a.len()],Some(record)));
        } else {
            assert(bounded(a[j],record.state));
            assert(rooted(b[i-a.len()],Some(record)));
        }
    }
}
}
