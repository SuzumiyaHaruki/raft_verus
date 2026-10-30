//! Conditional quorum dependency. Certificates/provenance are still obligations.
use vstd::prelude::*;
use super::state::tally;
verus! {
pub proof fn disjoint_bound(a: Seq<bool>, b: Seq<bool>, n: nat)
    requires n <= a.len(), n <= b.len(),
        forall|i: int| 0 <= i < n ==> !(a[i] && b[i]),
    ensures tally(a, n) + tally(b, n) <= n,
    decreases n,
{
    if n > 0 {
        disjoint_bound(a, b, (n - 1) as nat);
        assert(!(a[n as int - 1] && b[n as int - 1]));
    }
}
pub proof fn majority_overlap(a: Seq<bool>, b: Seq<bool>) -> (peer: int)
    requires a.len() == b.len(), tally(a, a.len()) > a.len() / 2,
        tally(b, b.len()) > b.len() / 2,
    ensures 0 <= peer < a.len(), a[peer], b[peer],
{
    if !(exists|i: int| 0 <= i < a.len() && a[i] && b[i]) {
        assert forall|i: int| 0 <= i < a.len() implies !(a[i] && b[i]) by {}
        disjoint_bound(a, b, a.len());
        assert(a.len() % 2 < 2);
        assert(a.len() == 2 * (a.len() / 2) + a.len() % 2);
        assert(false);
    }
    choose|i: int| 0 <= i < a.len() && a[i] && b[i]
}
// A vote ledger is historical, not a trusted network premise. A future global
// preservation proof must derive the certificates from durable runtime votes.
pub proof fn certified_leaders_agree(a: Seq<bool>, b: Seq<bool>, ledger: Map<(nat, nat), nat>,
    term: nat, first: nat, second: nat)
    requires a.len() == b.len(), tally(a, a.len()) > a.len() / 2,
        tally(b, b.len()) > b.len() / 2,
        forall|i: int| 0 <= i < a.len() && a[i] ==> ledger.contains_key((i as nat, term)) && ledger[(i as nat, term)] == first,
        forall|i: int| 0 <= i < b.len() && b[i] ==> ledger.contains_key((i as nat, term)) && ledger[(i as nat, term)] == second,
    ensures first == second,
{
    let peer = majority_overlap(a, b);
    assert(ledger[(peer as nat, term)] == first);
    assert(ledger[(peer as nat, term)] == second);
}
}
