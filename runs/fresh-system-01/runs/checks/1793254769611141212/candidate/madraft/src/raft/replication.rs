//! Executable log decisions; the async host transports their selected effects.
use vstd::prelude::*;
use super::election::Election;

verus! {
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Entry { pub term: u64, pub data: Vec<u8> }
pub struct Record { pub term: u64, pub data: Seq<u8> }
impl Entry {
    pub open spec fn view(&self) -> Record { Record { term: self.term, data: self.data@ } }
    pub fn copy(&self) -> (r: Self)
        ensures r.view() == self.view(),
    { Self { term: self.term, data: self.data.clone() } }
}
#[derive(Debug)]
pub struct Log {
    pub base: u64,
    pub base_term: u64,
    pub entries: Vec<Entry>,
    pub commit: u64,
    pub applied: u64,
}
pub open spec fn min64(a: u64, b: u64) -> u64 { if a < b { a } else { b } }
pub open spec fn records(v: Seq<Entry>) -> Seq<Record> {
    Seq::new(v.len(), |i: int| v[i].view())
}
pub struct Image { pub base: u64, pub base_term: u64, pub entries: Seq<Entry>, pub commit: u64, pub applied: u64 }
pub open spec fn valid_image(i: Image) -> bool {
    i.base <= i.applied <= i.commit && i.commit <= i.base + i.entries.len()
    && i.base + i.entries.len() <= u64::MAX
    && i.entries.len() <= usize::MAX
}
pub open spec fn tally(matched: Seq<u64>, upto: nat, index: u64, me: usize) -> nat
    recommends upto <= matched.len(),
    decreases upto,
{
    if upto == 0 { 0 } else { tally(matched, (upto-1) as nat, index, me)
        + if upto-1 == me || matched[upto as int-1] >= index { 1nat } else { 0nat } }
}
pub open spec fn append_allowed(i: Image, index: u64, entry: Entry) -> bool {
    index > i.base && index-i.base-1 <= i.entries.len() && index-i.base-1 < usize::MAX
    && (index > i.base+i.entries.len() || i.entries[index as int-i.base as int-1].term == entry.term || index > i.commit)
}
pub open spec fn appended(i: Image, index: u64, entry: Entry) -> Image {
    if !append_allowed(i, index, entry)
        || (index <= i.base+i.entries.len() && i.entries[index as int-i.base as int-1].term == entry.term) { i }
    else { Image { entries: i.entries.take(index as int-i.base as int-1).push(entry), ..i } }
}
impl Log {
    pub open spec fn image(&self) -> Image {
        Image { base: self.base, base_term: self.base_term, entries: self.entries@, commit: self.commit, applied: self.applied }
    }
    pub open spec fn valid(&self) -> bool { valid_image(self.image()) }
    pub fn new() -> (r: Self)
        ensures r.valid(), r.entries@ == Seq::<Entry>::empty(), r.commit == 0, r.applied == 0, r.base == 0, r.base_term == 0,
    { Self { base: 0, base_term: 0, entries: Vec::new(), commit: 0, applied: 0 } }
    pub fn restore(base: u64, base_term: u64, entries: Vec<Entry>, commit: u64) -> (r: Option<Self>)
        ensures r.is_some() ==> r.unwrap().valid() && r.unwrap().entries@ == entries@
            && r.unwrap().base == base && r.unwrap().base_term == base_term
            && r.unwrap().commit == commit && r.unwrap().applied == base,
    {
        if entries.len() as u128 + base as u128 > u64::MAX as u128
            || commit < base || commit as u128 > base as u128 + entries.len() as u128 { return None; }
        Some(Self { base, base_term, entries, commit, applied: base })
    }
    pub fn sync_election(&self, e: &mut Election)
        requires self.valid(), old(e).valid(),
        ensures e.valid(), e.last_index == self.base + self.entries.len(),
            e.last_term == if self.entries.len() == 0 {self.base_term} else {self.entries@.last().term},
            e.term == old(e).term, e.role == old(e).role, e.voted_for == old(e).voted_for,
            e.me == old(e).me, e.members == old(e).members,
            e.grants@ == old(e).grants@, e.grant_count == old(e).grant_count,
    {
        e.last_index = self.base + self.entries.len() as u64;
        e.last_term = if self.entries.len() == 0 { self.base_term } else { self.entries[self.entries.len()-1].term };
    }
    pub fn last_index(&self) -> (r: u64)
        requires self.valid(), ensures r == self.base + self.entries.len(),
    { self.base + self.entries.len() as u64 }
    pub fn term_at(&self, index: u64) -> (r: Option<u64>)
        requires self.valid(),
        ensures r == if index == self.base { Some(self.base_term) }
            else if self.base < index <= self.base+self.entries.len() {
                Some(self.entries@[index as int-self.base as int-1].term) } else { None },
    {
        if index == self.base { Some(self.base_term) }
        else if self.base < index && index <= self.last_index() {
            Some(self.entries[(index-self.base-1) as usize].term)
        } else { None }
    }
    pub fn propose(&mut self, term: u64, data: Vec<u8>) -> (r: Option<u64>)
        requires old(self).valid(),
        ensures self.valid(), self.base == old(self).base, self.base_term == old(self).base_term,
            self.commit == old(self).commit, self.applied == old(self).applied,
            r.is_some() ==> r.unwrap() == self.base+self.entries.len()
                && self.entries@ == old(self).entries@.push(Entry { term, data }),
            r.is_none() ==> self.entries@ == old(self).entries@,
    {
        if self.last_index() == u64::MAX || self.entries.len() == usize::MAX { return None; }
        self.entries.push(Entry { term, data });
        Some(self.last_index())
    }
    // One received entry at its original index. Matching entries preserve the
    // existing suffix; a differing term truncates only the uncommitted suffix.
    pub fn append_one(&mut self, index: u64, entry: Entry) -> (ok: bool)
        requires old(self).valid(),
        ensures self.valid(), self.base == old(self).base, self.base_term == old(self).base_term, ok == append_allowed(old(self).image(), index, entry),
            self.image() == appended(old(self).image(), index, entry),
            self.commit == old(self).commit, self.applied == old(self).applied,
            self.entries@.take(self.commit as int-self.base as int) == old(self).entries@.take(self.commit as int-self.base as int),
            !ok ==> self.entries@ == old(self).entries@,
            ok ==> self.base < index <= self.base+self.entries.len(),
    {
        let ghost before = self.image();
        if index <= self.base { return false; }
        let pos64 = index-self.base-1;
        if pos64 > self.entries.len() as u64 || pos64 >= usize::MAX as u64 { return false; }
        let pos = pos64 as usize;
        if pos < self.entries.len() {
            if self.entries[pos].term == entry.term { return true; }
            if index <= self.commit { return false; }
            self.entries.truncate(pos);
        }
        proof { assert(self.entries@ =~= before.entries.take(pos as int)); }
        self.entries.push(entry);
        true
    }
    pub fn follower_commit(&mut self, advertised: u64, matched_through: u64)
        requires old(self).valid(),
        ensures self.valid(), self.base == old(self).base, self.base_term == old(self).base_term, self.entries@ == old(self).entries@,
            self.applied == old(self).applied, self.commit >= old(self).commit,
            self.commit == if min64(min64(advertised, matched_through), (old(self).base + old(self).entries.len()) as u64) > old(self).commit {
                min64(min64(advertised, matched_through), (old(self).base + old(self).entries.len()) as u64)
            } else { old(self).commit },
    {
        let bound = if advertised < matched_through { advertised } else { matched_through };
        let bound = if bound < self.last_index() { bound } else { self.base + self.entries.len() as u64 };
        if bound > self.commit { self.commit = bound; }
    }
    pub fn leader_commit(&mut self, term: u64, me: usize, matched: &Vec<u64>)
        requires old(self).valid(),
        ensures self.valid(), self.base == old(self).base, self.base_term == old(self).base_term, self.entries@ == old(self).entries@,
            self.applied == old(self).applied, self.commit >= old(self).commit,
            self.commit > old(self).commit ==> self.entries@[self.commit as int-self.base as int-1].term == term
                && tally(matched@, matched.len() as nat, self.commit, me) > matched.len() / 2,
    {
        if matched.len() == 0 || me >= matched.len() { return; }
        let mut index = self.entries.len();
        while self.base+index as u64 > self.commit
            invariant self.valid(), self.entries@ == old(self).entries@,
                self.applied == old(self).applied, self.commit == old(self).commit,
                self.commit <= self.base+index, index <= self.entries.len(), self.base == old(self).base, self.base_term == old(self).base_term, me < matched.len(),
            decreases index,
        {
            if self.entries[index - 1].term == term {
                let absolute = self.base+index as u64;
                let mut count: usize = 0;
                let mut peer: usize = 0;
                while peer < matched.len()
                    invariant peer <= matched.len(), count <= peer,
                        count == tally(matched@, peer as nat, absolute, me),
                    decreases matched.len() - peer,
                {
                    if peer == me || matched[peer] >= absolute { count += 1; }
                    peer += 1;
                }
                if count > matched.len() / 2 { self.commit = self.base+index as u64; return; }
            }
            index -= 1;
        }
    }
    pub fn compact(&mut self, index: u64) -> (ok: bool)
        requires old(self).valid(),
        ensures self.valid(), self.commit == old(self).commit, self.applied == old(self).applied,
            !ok ==> self.image() == old(self).image(),
            ok ==> self.base == index && self.entries@ == old(self).entries@.skip((index-old(self).base) as int)
                && self.base_term == old(self).entries@[index as int-old(self).base as int-1].term,
    {
        if index <= self.base || index > self.applied { return false; }
        let count = (index-self.base) as usize;
        let term = self.entries[count-1].term;
        let suffix = self.entries.split_off(count);
        self.entries = suffix;
        self.base = index;
        self.base_term = term;
        true
    }
    pub fn install(&mut self, index: u64, term: u64) -> (ok: bool)
        requires old(self).valid(),
        ensures self.valid(), !ok ==> self.image() == old(self).image(),
            ok ==> self.base == index && self.base_term == term && self.commit == index && self.applied == index,
    {
        if index <= self.commit { return false; }
        if self.term_at(index) == Some(term) {
            let suffix = self.entries.split_off((index-self.base) as usize);
            self.entries = suffix;
        } else { self.entries = Vec::new(); }
        self.base = index;
        self.base_term = term;
        self.commit = index;
        self.applied = index;
        true
    }
    pub fn next_application(&mut self) -> (r: Option<(u64, Entry)>)
        requires old(self).valid(),
        ensures self.valid(), self.base == old(self).base, self.base_term == old(self).base_term, self.entries@ == old(self).entries@, self.commit == old(self).commit,
            r.is_none() ==> self.applied == old(self).applied,
            r.is_some() ==> self.applied == old(self).applied + 1 && r.unwrap().0 == self.applied
                && r.unwrap().1.view() == self.entries@[self.applied as int-self.base as int-1].view(),
    {
        if self.applied >= self.commit { return None; }
        let index = self.applied + 1;
        let entry = self.entries[(index-self.base-1) as usize].copy();
        self.applied = index;
        Some((index, entry))
    }
}
}
