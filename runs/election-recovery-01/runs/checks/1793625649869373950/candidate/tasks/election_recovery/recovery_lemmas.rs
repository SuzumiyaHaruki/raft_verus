//! OPEN candidate obligations. Invent the cross-instance invariant and prove it.
use crate::recovery_network::*;
use crate::vote_api::*;
use vstd::prelude::*;
verus! {
pub closed spec fn recovery_inv(t:Trace) -> bool { false }
pub proof fn initialize(t:Trace) requires boot(t), ensures recovery_inv(t), { assert(false); }
pub proof fn preserve(a:Trace,b:Trace,s:Step)
    requires recovery_inv(a),shape(a),enabled(a,s),b==step_after(a,s),
    ensures recovery_inv(b),
{ assert(false); }
pub proof fn conclude(t:Trace)
    requires recovery_inv(t),shape(t),
    ensures recovered_from_own_write(t),unique(t.choices),provenance(t),historical_safety(t),
{ assert(false); }
}
