//! Consequences of the invariant maintained by the actual executable Core.
use crate::{vote_api::*, vote_core::{Core,bounded,above}};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(tracked c: &Core) ensures unique(c.choices()) { c.facts(); }
pub proof fn restart_composition(tracked before: &Core, tracked after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.choices() + after.choices()),
{
    before.facts(); after.facts();
    let a = before.choices(); let b = after.choices();
    assert forall|i:int,j:int| 0 <= i < (a+b).len() && 0 <= j < (a+b).len()
        && (#[trigger] (a+b)[i]).node == (#[trigger] (a+b)[j]).node && (a+b)[i].term == (a+b)[j].term
        implies (a+b)[i].candidate == (a+b)[j].candidate by {
        if i < a.len() && j < a.len() { assert(a[i].candidate == a[j].candidate); }
        else if i >= a.len() && j >= a.len() { assert(b[i-a.len()].candidate == b[j-a.len()].candidate); }
        else if i < a.len() {
            assert(a[i].term <= record.state.term);
            assert(b[j-a.len()].term >= record.state.term);
            assert(record.state.voted_for == Some(a[i].candidate));
        } else {
            assert(a[j].term <= record.state.term);
            assert(b[i-a.len()].term >= record.state.term);
            assert(record.state.voted_for == Some(a[j].candidate));
        }
    }
}
}
