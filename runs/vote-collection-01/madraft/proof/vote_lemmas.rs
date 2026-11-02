//! Student proof entry. No implementation-specific invariant is supplied.
use crate::{vote_api::*, vote_core::Core};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(tracked c: &Core) ensures unique(c.choices()) {
    c.history();
}
pub proof fn restart_composition(tracked before: &Core, tracked after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.choices() + after.choices()),
{
    before.history(); after.history();
    assert forall|i:int,j:int| 0<=i<(before.choices()+after.choices()).len() && 0<=j<(before.choices()+after.choices()).len()
        && (#[trigger] (before.choices()+after.choices())[i]).node == (#[trigger] (before.choices()+after.choices())[j]).node
        && (before.choices()+after.choices())[i].term == (before.choices()+after.choices())[j].term
        implies (before.choices()+after.choices())[i].candidate == (before.choices()+after.choices())[j].candidate by {
        if i<before.choices().len() && j>=before.choices().len() {
            assert(after.choices()[j-before.choices().len()].term>=record.state.term);
        } else if j<before.choices().len() && i>=before.choices().len() {
            assert(after.choices()[i-before.choices().len()].term>=record.state.term);
        }
    }
}
}

#[cfg(any(vc_provenance,vc_nonempty))]
#[path="vote_sources.rs"] mod sources;
#[cfg(any(vc_provenance,vc_nonempty))]
pub use sources::*;
