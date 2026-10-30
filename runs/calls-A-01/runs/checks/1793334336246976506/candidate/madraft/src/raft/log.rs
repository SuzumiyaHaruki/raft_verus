use vstd::prelude::*;
verus! {
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(super) struct Entry { pub(super) term: u64, pub(super) data: Vec<u8> }

pub(super) fn fresh(candidate_term: u64, candidate_index: u64, local_term: u64, local_index: u64) -> (ok: bool)
    ensures ok == (candidate_term > local_term || (candidate_term == local_term && candidate_index >= local_index)),
{
    candidate_term > local_term || (candidate_term == local_term && candidate_index >= local_index)
}

pub(super) fn append_command(log: &mut Vec<Entry>, term: u64, data: Vec<u8>)
    requires old(log).len() < usize::MAX,
    ensures log@.len() == old(log)@.len() + 1,
        log@.take(old(log)@.len() as int) == old(log)@,
        log@[old(log)@.len() as int].term == term,
        log@[old(log)@.len() as int].data@ == data@,
{
    log.push(Entry { term, data });
}

pub(super) fn truncate_suffix(log: &mut Vec<Entry>, keep: usize)
    requires keep <= old(log).len(),
    ensures log@ == old(log)@.take(keep as int),
{
    log.truncate(keep);
}
// Bounds checks are executable: malformed indices leave the physical log intact.
pub(super) fn compact(log: &mut Vec<Entry>, remove: usize) -> (ok: bool)
    ensures ok == (remove <= old(log).len()),
        ok ==> log@ == old(log)@.subrange(remove as int, old(log)@.len() as int),
        !ok ==> log@ == old(log)@,
{
    if remove > log.len() { return false; }
    let mut i = 0usize;
    while i < remove
        invariant i <= remove, remove <= old(log).len(),
            log@ == old(log)@.subrange(i as int, old(log)@.len() as int),
        decreases remove - i,
    {
        log.remove(0);
        i += 1;
    }
    true
}

// offset is relative to the first retained entry; committed is the count of
// retained entries already committed, not an application/configuration number.
pub(super) fn reconcile(log: &mut Vec<Entry>, offset: usize, committed: usize, entry: Entry) -> (ok: bool)
    requires committed <= old(log).len(),
    ensures !ok ==> log@ == old(log)@,
        ok ==> offset < log.len() && log@[offset as int].term == entry.term
            && log@[offset as int].data@ == entry.data@,
        log@.take(committed as int) == old(log)@.take(committed as int),
        log.len() >= committed,
{
    if offset > log.len() { return false; }
    if offset < log.len() && log[offset].term == entry.term {
        return super::observation::same_bytes(&log[offset].data, &entry.data);
    }
    if offset < committed || offset == usize::MAX { return false; }
    log.truncate(offset);
    log.push(entry);
    true
}

pub(super) fn last(base: u64, log: &Vec<Entry>) -> (result: Option<u64>)
    ensures match result { Some(index) => index == base + log.len(), None => base + log.len() > u64::MAX },
{
    if log.len() as u128 > u64::MAX as u128 { return None; }
    let len = log.len() as u64;
    if base > u64::MAX - len { None } else { Some(base + len) }
}

pub(super) fn next_application(base: u64, log: &Vec<Entry>, applied: u64, committed: u64) -> (result: Option<(u64, Vec<u8>)>)
    ensures match result {
        Some((index, data)) => index == applied + 1 && base < index <= committed
            && committed <= base + log.len()
            && data@ == log@[(index - base - 1) as int].data@,
        None => applied < base || applied >= committed || committed > base + log.len()
            || base + log.len() > u64::MAX,
    },
{
    let end = match last(base, log) { Some(index) => index, None => return None };
    if applied < base || applied >= committed || committed > end { return None; }
    let index = applied + 1;
    let offset = (index - base - 1) as usize;
    Some((index, log[offset].data.clone()))
}

}
