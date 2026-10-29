//! Executable log mutations shared with the formal development.
use vstd::prelude::*;
verus! {
#[derive(Clone, Debug)]
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
pub struct Entry { pub term: u64, pub command: Vec<u8> }

pub fn matches(log: &Vec<Entry>, prev: usize, term: u64) -> (ok: bool)
    ensures ok == (prev == 0 || (0 < prev <= log.len() && log@[prev as int - 1].term == term)),
{
    prev == 0 || (prev <= log.len() && log[prev - 1].term == term)
}
// A matching term is kept. Equality of its command is a GLOBAL provenance
// obligation, not a precondition used to hide arbitrary received messages.
pub fn insert(log: &mut Vec<Entry>, at: usize, entry: Entry) -> (ok: bool)
    ensures
        ok == (at <= old(log).len() && at < usize::MAX),
        !ok ==> log@ == old(log)@,
        ok && at < old(log).len() && old(log)@[at as int].term == entry.term ==> log@ == old(log)@,
        ok && (at == old(log).len() || old(log)@[at as int].term != entry.term) ==>
            log@ == old(log)@.take(at as int).push(entry),
{
    if at > log.len() || at == usize::MAX { return false; }
    if at < log.len() {
        if log[at].term == entry.term { return true; }
        log.truncate(at);
    }
    log.push(entry);
    true
}
pub fn propose(log: &mut Vec<Entry>, entry: Entry) -> (ok: bool)
    ensures ok == (old(log).len() < usize::MAX),
        ok ==> log@ == old(log)@.push(entry), !ok ==> log@ == old(log)@,
{
    if log.len() == usize::MAX { return false; }
    log.push(entry); true
}
pub fn commit_candidate(current: u64, index: u64, entry_term: u64, term: u64,
    replicated: usize, members: usize) -> (next: u64)
    ensures next == (if index > current && entry_term == term && replicated > members / 2 { index } else { current }),
        next >= current,
{
    if index > current && entry_term == term && replicated > members / 2 { index } else { current }
}
}
