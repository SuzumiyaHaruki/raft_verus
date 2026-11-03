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
    before.history();after.history();
    let a=before.choices();let b=after.choices();let joined=a+b;
    assert forall|i:int,j:int| 0<=i<joined.len() && 0<=j<joined.len()
        && (#[trigger] joined[i]).node==(#[trigger] joined[j]).node && joined[i].term==joined[j].term
        implies joined[i].candidate==joined[j].candidate by {
        if i<a.len() && j<a.len() {assert(joined[i]==a[i]);assert(joined[j]==a[j]);}
        else if i>=a.len() && j>=a.len() {assert(joined[i]==b[i-a.len()]);assert(joined[j]==b[j-a.len()]);}
        else if i<a.len() {
            assert(joined[i]==a[i]);assert(joined[j]==b[j-a.len()]);
            assert(a[i].term<=record.state.term);assert(b[j-a.len()].term>=record.state.term);
            assert(a[i].term==record.state.term);
            assert(record.state.voted_for==Some(a[i].candidate));
        } else {
            assert(joined[j]==a[j]);assert(joined[i]==b[i-a.len()]);
            assert(a[j].term<=record.state.term);assert(b[i-a.len()].term>=record.state.term);
            assert(a[j].term==record.state.term);
            assert(record.state.voted_for==Some(a[j].candidate));
        }
    }
}
}

#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty))]
#[path="network_invariant.rs"] mod network_invariant;
// Checked local Core history is lifted to actual network evidence.
#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty))]
use crate::vote_network::*;
#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty))]
verus! {
pub closed spec fn network_inv(t: Trace) -> bool { network_invariant::inv(t) }
pub proof fn network_initial(t: Trace)
    requires boot(t), ensures network_inv(t),
{ network_invariant::initialize(t); }
pub proof fn network_step(before: Trace, after: Trace, event: Step)
    requires network_inv(before), shape(before), enabled(before,event), after == step_after(before,event),
    ensures network_inv(after),
{ network_invariant::preserve(before,after,event); }
pub proof fn network_conclusion(t: Trace)
    requires network_inv(t), shape(t),
    ensures sourced(t), choice_sourced(t), elections_justified(t), historical_safety(t),
{ network_invariant::conclude(t); }
}
