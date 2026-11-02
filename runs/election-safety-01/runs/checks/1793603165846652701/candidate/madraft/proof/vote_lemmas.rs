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
    let a=before.choices();let b=after.choices();
    assert forall|i:int,j:int| 0<=i<(a+b).len() && 0<=j<(a+b).len() && (#[trigger] (a+b)[i]).node==(#[trigger] (a+b)[j]).node && (a+b)[i].term==(a+b)[j].term implies (a+b)[i].candidate==(a+b)[j].candidate by {
        if i<a.len() && j<a.len() {assert(a[i].candidate==a[j].candidate);}
        else if i>=a.len() && j>=a.len() {assert(b[i-a.len()].candidate==b[j-a.len()].candidate);}
        else if i<a.len() {assert(a[i].term==record.state.term);}
        else {assert(a[j].term==record.state.term);}
    }
}
}

#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty))]
#[path="network_proof.rs"] mod network_proof;
#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty))]
pub use network_proof::*;
