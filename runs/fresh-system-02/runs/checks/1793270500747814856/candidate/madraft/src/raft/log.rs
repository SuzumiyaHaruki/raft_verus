//! Executable log mutations shared with the formal development.
use vstd::prelude::*;
use super::election::*;
use super::quorum;
verus! {
pub struct Core { pub group: u64, pub node: usize, pub members:usize,pub votes:Vec<bool>, pub election: Election, pub log: Vec<Entry>, pub commit: u64, pub applied: u64 }
pub fn new_core(group: u64,node: usize,members:usize) -> (c: Core)
    ensures c.group == group, c.node == node, c.members == members,c.votes.len()==members,forall|j:int|0<=j<c.votes.len() ==> !#[trigger] c.votes@[j], c.election == fresh(), c.log.len() == 0, c.commit == 0, c.applied == 0,
{ Core { group,node,members,votes:quorum::empty(members),election: new(), log: Vec::new(), commit: 0, applied: 0 } }
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
pub open spec fn inserted(log: Seq<Entry>, at: nat, entry: Entry) -> Seq<Entry> {
    if at > log.len() || at >= usize::MAX { log }
    else if at < log.len() && log[at as int].term == entry.term { log }
    else { log.take(at as int).push(entry) }
}
pub fn next_application(applied: u64, committed: u64, len: usize) -> (index: Option<u64>)
    ensures index == (if applied < committed && committed <= len {
        Some((applied + 1) as u64) } else { None }),
        index.is_some() ==> 1 <= index.unwrap() <= len,
{
    if applied < committed && committed <= len as u64 { Some(applied+1) } else { None }
}
pub proof fn insertion_keeps_prefix(log: Seq<Entry>,at: nat,entry: Entry,k: nat)
    requires k <= at <= log.len(), at < usize::MAX,
    ensures inserted(log,at,entry).take(k as int) == log.take(k as int),
{
    if !(at < log.len() && log[at as int].term == entry.term) {
        assert(log.take(at as int).push(entry).take(k as int) =~= log.take(k as int));
    }
}

}
