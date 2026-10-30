//! Actual log mutations, shared by Rust execution and Verus.
use vstd::prelude::*;
verus! {
#[cfg_attr(not(verus_keep_ghost), derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize))]
pub(super) struct LogEntry { pub(super) term: u64, pub(super) data: Vec<u8> }

pub(super) fn fresh_log() -> (log: Vec<LogEntry>)
    ensures log.len() == 1, log@[0].term == 0, log@[0].data@ == Seq::<u8>::empty(),
{
    let mut log = Vec::new();
    log.push(LogEntry { term: 0, data: Vec::new() });
    log
}

pub(super) fn same(a: &LogEntry, b: &LogEntry) -> (yes: bool)
    ensures yes == (a.term == b.term && a.data@ == b.data@),
{
    if a.term != b.term || a.data.len() != b.data.len() { return false; }
    let mut i = 0;
    while i < a.data.len()
        invariant i <= a.data.len(), a.data.len() == b.data.len(),
            forall|j: int| 0 <= j < i ==> a.data@[j] == b.data@[j],
        decreases a.data.len() - i,
    {
        if a.data[i] != b.data[i] { return false; }
        i += 1;
    }
    proof { assert(a.data@ =~= b.data@); }
    true
}

// No unchecked helper preconditions: malformed or exhausted boundaries reject.
// Real role authorization is the caller's State::is_leader result; its
// constructor/recovery composition is still a separate proof obligation.
pub(super) fn propose(log: &mut Vec<LogEntry>, base: usize, term: u64, leader: bool, data: Vec<u8>) -> (accepted: bool)
    ensures accepted ==> leader && log.len() == old(log).len() + 1
        && log@.take(old(log).len() as int) == old(log)@
        && log@[old(log).len() as int].term == term
        && log@[old(log).len() as int].data@ == data@,
        !accepted ==> log@ == old(log)@,
{
    if !leader || log.len() == 0 || log.len() >= usize::MAX - base { return false; }
    log.push(LogEntry { term, data });
    true
}

pub(super) fn truncate_conflict(log: &mut Vec<LogEntry>, base: usize, committed: usize, offset: usize, incoming: &LogEntry) -> (changed: bool)
    ensures !changed ==> log@ == old(log)@,
        changed ==> offset < old(log).len() && log@ == old(log)@.take(offset as int)
            && base + offset > committed && old(log)@[offset as int].term != incoming.term,
        // Exact retained content, including each command's bytes.
        forall|i: int| 0 <= i < log.len() ==> log@[i] == old(log)@[i],
{
    if offset >= log.len() || base > usize::MAX - offset { return false; }
    if base + offset <= committed || log[offset].term == incoming.term { return false; }
    log.truncate(offset);
    true
}

pub(super) fn append_received(log: &mut Vec<LogEntry>, entry: LogEntry) -> (accepted: bool)
    ensures accepted ==> log@ == old(log)@.push(entry),
        !accepted ==> log@ == old(log)@,
{
    if log.len() == usize::MAX { return false; }
    log.push(entry);
    true
}
// Actual proposal caller: derives helper inputs from real consensus state.
pub(super) fn propose_from_state(state: &super::state::State, log: &mut Vec<LogEntry>, base: usize, data: Vec<u8>) -> (index: Option<usize>)
    ensures index.is_some() ==> state.role is Leader
        && index.unwrap() == base + old(log).len()
        && log.len() == old(log).len() + 1
        && log@.take(old(log).len() as int) == old(log)@
        && log@[old(log).len() as int].term == state.term
        && log@[old(log).len() as int].data@ == data@,
        index.is_none() ==> log@ == old(log)@,
{
    if log.len() == 0 || log.len() >= usize::MAX - base { return None; }
    let index = base + log.len();
    let leader = state.is_leader();
    if propose(log, base, state.term, leader, data) { Some(index) } else { None }
}

// Actual received-entry call composition. The async handler separately checks
// term, predecessor and genuine-message provenance; those remain OPEN.
pub(super) fn receive_entry(log: &mut Vec<LogEntry>, base: usize, committed: usize, index: usize, entry: LogEntry) -> (accepted: bool)
    ensures accepted ==> index >= base && index - base < log.len()
        && log@[index as int-base as int].term == entry.term
        && log@[index as int-base as int].data@ == entry.data@,
{
    if index < base { return false; }
    let offset = index - base;
    if offset > log.len() { return false; }
    if offset < log.len() {
        if same(&log[offset], &entry) { return true; }
        if !truncate_conflict(log, base, committed, offset, &entry) { return false; }
    }
    append_received(log, entry)
}

}
