//! Student proof entry. No implementation-specific invariant is supplied.
use crate::{vote_api::*, vote_core::Core};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(c: &Core) ensures unique(c.replies()) {
    assert(false); // OPEN: generate an invariant of the actual executable core.
}
#[cfg(any(rv_confirmed, rv_pending))]
pub proof fn restart_composition(before: &Core, after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.replies() + after.replies()),
{
    assert(false); // OPEN: preserved vote constrains every subsequent actual output.
}
}
