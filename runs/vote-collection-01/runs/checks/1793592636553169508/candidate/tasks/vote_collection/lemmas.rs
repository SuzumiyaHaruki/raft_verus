//! Student proof entry. No implementation-specific invariant is supplied.
use crate::{vote_api::*, vote_core::Core};
use vstd::prelude::*;
verus! {
pub proof fn uniqueness(tracked c: &Core) ensures unique(c.choices()) {
    assert(false); // OPEN: generate an invariant of the actual executable core.
}
pub proof fn restart_composition(tracked before: &Core, tracked after: &Core, record: Record)
    requires before.view().state == record.state, before.view().next == record.op + 1,
        after.base() == Some(record), before.view().config == after.view().config,
    ensures unique(before.choices() + after.choices()),
{
    assert(false); // OPEN: preserved vote constrains every subsequent actual output.
}
}

// Candidate-designed source invariant and preservation proof. No answer supplied.
#[cfg(any(vc_provenance,vc_nonempty))]
use crate::vote_network::*;
#[cfg(any(vc_provenance,vc_nonempty))]
verus! {
pub closed spec fn network_inv(t: Trace) -> bool { false } // OPEN: choose your invariant.
pub proof fn network_initial(t: Trace)
    requires boot(t),
    ensures network_inv(t),
{ assert(false); }
pub proof fn network_step(before: Trace, after: Trace, event: Step)
    requires network_inv(before), shape(before), enabled(before,event), after == step_after(before,event),
    ensures network_inv(after),
{ assert(false); }
pub proof fn network_conclusion(t: Trace)
    requires network_inv(t), shape(t),
    ensures sourced(t),
{ assert(false); }
}
