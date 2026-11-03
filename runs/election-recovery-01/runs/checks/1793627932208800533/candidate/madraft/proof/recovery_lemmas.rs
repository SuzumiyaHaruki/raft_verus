//! OPEN candidate obligations. Invent the cross-instance invariant and prove it.
use crate::recovery_network::*;
use crate::vote_api::*;
use vstd::prelude::*;
#[path="recovery_invariant.rs"] mod recovery_invariant;
#[path="recovery_steps.rs"] mod recovery_steps;
verus! {
pub closed spec fn recovery_inv(t:Trace) -> bool { recovery_invariant::inv(t) }
pub proof fn initialize(t:Trace) requires boot(t),t.net.nodes.len()<=usize::MAX, ensures recovery_inv(t), { recovery_invariant::initialize(t); }
pub proof fn preserve(a:Trace,b:Trace,s:Step)
    requires recovery_inv(a),shape(a),enabled(a,s),b==step_after(a,s),shape(b),a.frames.len()<usize::MAX,
    ensures recovery_inv(b),
{ recovery_steps::preserve(a,b,s); }
pub proof fn conclude(t:Trace)
    requires recovery_inv(t),shape(t),
    ensures recovered_from_own_write(t),unique(t.choices),provenance(t),historical_safety(t),
{ recovery_invariant::conclude(t); }
}
