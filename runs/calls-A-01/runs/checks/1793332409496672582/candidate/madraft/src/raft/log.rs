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
}
