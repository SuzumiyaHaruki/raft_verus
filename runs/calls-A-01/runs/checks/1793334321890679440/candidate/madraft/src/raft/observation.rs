use vstd::prelude::*;
verus! {
#[derive(Debug)]
pub(super) struct Receipt { pub(super) term: u64, pub(super) index: u64, pub(super) data: Vec<u8> }
pub(super) open spec fn matches(r: Receipt, term: u64, index: u64, data: Seq<u8>) -> bool {
    r.term == term && r.index == index && r.data@ == data
}
pub(super) fn same_bytes(a: &Vec<u8>, b: &Vec<u8>) -> (same: bool)
    ensures same == (a@ == b@),
{
    if a.len() != b.len() { return false; }
    let mut i = 0usize;
    while i < a.len()
        invariant a.len() == b.len(), i <= a.len(), forall|j: int| 0 <= j < i ==> a@[j] == b@[j],
        decreases a.len() - i,
    {
        if a[i] != b[i] { return false; }
        i += 1;
    }
    proof { assert(a@ =~= b@); }
    true
}
pub(super) fn record(pending: &mut Vec<Receipt>, term: u64, index: u64, data: Vec<u8>)
    requires old(pending).len() < usize::MAX,
    ensures pending@.len() == old(pending)@.len() + 1,
        pending@.take(old(pending)@.len() as int) == old(pending)@,
        matches(pending@[old(pending)@.len() as int], term, index, data@),
{
    pending.push(Receipt { term, index, data });
}
pub(super) fn claim(pending: &mut Vec<Receipt>, term: u64, index: u64, data: &Vec<u8>) -> (accepted: bool)
    ensures accepted == (exists|i: int| 0 <= i < old(pending)@.len() && matches(old(pending)@[i], term, index, data@)),
        accepted ==> exists|i: int| 0 <= i < old(pending)@.len() && matches(old(pending)@[i], term, index, data@)
            && pending@ == old(pending)@.remove(i),
        !accepted ==> pending@ == old(pending)@,
        accepted ==> pending@.len() + 1 == old(pending)@.len(),
{
    let mut i = 0usize;
    while i < pending.len()
        invariant pending@ == old(pending)@, i <= pending.len(),
            forall|j: int| 0 <= j < i ==> !matches(pending@[j], term, index, data@),
        decreases pending.len() - i,
    {
        if pending[i].term == term && pending[i].index == index && same_bytes(&pending[i].data, data) {
            proof { assert(matches(pending@[i as int], term, index, data@)); }
            pending.remove(i);
            return true;
        }
        i += 1;
    }
    false
}
}
