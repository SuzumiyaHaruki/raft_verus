//! Counts distinct member slots and selects only current-term majority commits.
use vstd::prelude::*;
use super::{log::Entry, state};
verus! {
pub open spec fn votes(v: Seq<bool>, n: nat) -> nat
    recommends n <= v.len(),
    decreases n,
{
    if n == 0 { 0 } else { votes(v, (n - 1) as nat) + if v[n as int - 1] { 1nat } else { 0nat } }
}
pub open spec fn acks(v: Seq<u64>, n: nat, index: u64) -> nat
    recommends n <= v.len(),
    decreases n,
{
    if n == 0 { 0 } else { acks(v, (n - 1) as nat, index) + if v[n as int - 1] >= index { 1nat } else { 0nat } }
}
pub fn count_votes(v: &Vec<bool>) -> (count: usize)
    ensures count == votes(v@, v.len() as nat), count <= v.len(),
{
    let mut count = 0;
    let mut i = 0;
    while i < v.len()
        invariant i <= v.len(), count <= i, count == votes(v@, i as nat),
        decreases v.len() - i,
    {
        if v[i] { count = count + 1; }
        i = i + 1;
    }
    count
}
pub fn count_acks(v: &Vec<u64>, index: u64) -> (count: usize)
    ensures count == acks(v@, v.len() as nat, index), count <= v.len(),
{
    let mut count = 0;
    let mut i = 0;
    while i < v.len()
        invariant i <= v.len(), count <= i, count == acks(v@, i as nat, index),
        decreases v.len() - i,
    {
        if v[i] >= index { count = count + 1; }
        i = i + 1;
    }
    count
}
pub fn reset_votes(v: &mut Vec<bool>, me: usize)
    requires me < old(v).len(),
    ensures v.len() == old(v).len(), forall|i: int| 0 <= i < v.len() ==> v@[i] == (i == me),
{
    let mut i = 0;
    while i < v.len()
        invariant i <= v.len(), v.len() == old(v).len(),
            forall|j: int| 0 <= j < i ==> v@[j] == (j == me),
        decreases v.len() - i,
    {
        v.set(i, i == me);
        i = i + 1;
    }
}
pub fn record_vote(v: &mut Vec<bool>, member: usize) -> (ok: bool)
    ensures
        v.len() == old(v).len(),
        ok == (member < v.len()),
        ok ==> v@ == old(v)@.update(member as int, true),
        !ok ==> v@ == old(v)@,
{
    if member >= v.len() { return false; }
    v.set(member, true);
    true
}

pub fn select_commit(log: &Vec<Entry>, base: u64, commit: u64, term: u64,
    matched: &Vec<u64>, members: usize) -> (result: u64)
    requires base <= commit <= base as int + log.len() <= u64::MAX, matched.len() == members,
    ensures
        commit <= result <= base as int + log.len(),
        result > commit ==> log@[(result - base - 1) as int].term == term
            && acks(matched@, matched.len() as nat, result) > members / 2
            && members > 0,
{
    let end = base + log.len() as u64;
    let mut index = end;
    while index > commit
        invariant commit <= index <= end, end == base as int + log.len(), base <= commit,
        decreases index - commit,
    {
        let count = count_acks(matched, index);
        let entry_term = log[(index - base - 1) as usize].term;
        if state::commit_authorized(state::Role::Leader, term, entry_term,
            index, commit, end, count, members) { return index; }
        index = index - 1;
    }
    commit
}

// Explicitly conditional: a real election proof must establish disjoint votes
// for different candidates from initialized and preserved durable vote state.
pub proof fn disjoint_vote_bound(a: Seq<bool>, b: Seq<bool>, n: nat)
    requires n <= a.len(), a.len() == b.len(),
        forall|i: int| 0 <= i < n ==> !(a[i] && b[i]),
    ensures votes(a, n) + votes(b, n) <= n,
    decreases n,
{
    if n > 0 { disjoint_vote_bound(a, b, (n - 1) as nat); }
}
pub proof fn majority_intersection(a: Seq<bool>, b: Seq<bool>)
    requires a.len() == b.len(), votes(a, a.len()) > a.len() / 2, votes(b, b.len()) > b.len() / 2,
    ensures exists|i: int| 0 <= i < a.len() && a[i] && b[i],
{
    if !(exists|i: int| 0 <= i < a.len() && a[i] && b[i]) {
        disjoint_vote_bound(a, b, a.len());
    }
}

pub fn fill(v: &mut Vec<u64>, value: u64)
    ensures v.len() == old(v).len(), forall|i: int| 0 <= i < v.len() ==> v@[i] == value,
{
    let mut i = 0;
    while i < v.len()
        invariant i <= v.len(), v.len() == old(v).len(), forall|j: int| 0 <= j < i ==> v@[j] == value,
        decreases v.len() - i,
    {
        v.set(i, value);
        i = i + 1;
    }
}

}
