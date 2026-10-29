//! Synchronous Raft engine. Networking/storage are confined to raft.rs.
//! Local functional contracts and the representation invariant are verified.
//! Cluster histories, host refinement and global safety remain open.
use super::{state::{self, Role, State}, log, quorum, cursor};
pub use log::Entry;
use vstd::prelude::*;

verus! {

#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct Durable {
    pub term: u64,
    pub voted_for: Option<usize>,
    pub base: u64,
    pub base_term: u64,
    pub log: Vec<Entry>,
}

#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct Vote {
    pub term: u64, pub candidate: usize, pub last_index: u64, pub last_term: u64,
}
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct VoteReply { pub term: u64, pub granted: bool }
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct Append {
    pub term: u64, pub leader: usize, pub prev: u64, pub prev_term: u64,
    pub entries: Vec<Entry>, pub commit: u64,
}
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct AppendReply { pub term: u64, pub success: bool, pub next: u64 }
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct Install {
    pub term: u64, pub leader: usize, pub index: u64, pub last_term: u64, pub data: Vec<u8>,
}
#[cfg_attr(not(verus_keep_ghost), derive(Clone, serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct InstallReply { pub term: u64, pub accepted: bool }

pub enum Replication { Append(Append), Snapshot(Install) }
pub enum Applied { Command { index: u64, data: Vec<u8> }, Snapshot { index: u64, term: u64, data: Vec<u8> } }

pub struct Core {
    pub state: State,
    pub me: usize,
    pub members: usize,
    pub base: u64,
    pub base_term: u64,
    pub log: Vec<Entry>,
    pub snapshot: Vec<u8>,
    pub commit: u64,
    pub applied: u64,
    pub leader: usize,
    pub reset_timer: bool,
    pub dirty: bool,
    pub next: Vec<u64>,
    pub matched: Vec<u64>,
    pub votes: Vec<bool>,
    pub pending_snapshot: bool,
    pub awaiting_snapshot: bool,
}

impl Core {
    pub open spec fn wf(&self) -> bool {
        self.members > 0 && self.me < self.members && self.leader < self.members
        && self.next.len() == self.members && self.matched.len() == self.members && self.votes.len() == self.members
        && self.base <= self.applied <= self.commit <= self.base as int + self.log.len() <= u64::MAX
        && self.base_term <= self.state.term && log::bounded(self.log@, self.state.term)
        && (self.state.voted_for.is_some() ==> self.state.voted_for.unwrap() < self.members)
        && ((self.state.role is Candidate || self.state.role is Leader) ==> self.state.voted_for == Some(self.me))
        && (self.state.role is Leader ==> quorum::votes(self.votes@, self.votes.len() as nat) > self.members / 2)
        && (self.state.role is Leader ==> forall|i: int| 0 <= i < self.members ==>
            1 <= #[trigger] self.next@[i] <= if self.base as int + self.log.len() == u64::MAX { u64::MAX as int } else { self.base as int + self.log.len() + 1 })
        && (self.state.role is Leader ==> forall|i: int| 0 <= i < self.members ==> #[trigger] self.matched@[i] <= self.base as int + self.log.len())
        && (self.pending_snapshot ==> self.awaiting_snapshot)
    }
    pub open spec fn persistent_view(&self) -> (u64, Option<usize>, u64, u64, Seq<(u64, Seq<u8>)>, Seq<u8>) {
        (self.state.term, self.state.voted_for, self.base, self.base_term, log::log_view(self.log@), self.snapshot@)
    }

    pub fn new(me: usize, members: usize) -> (r: Self)
        requires members > 0, me < members,
        ensures r.wf(), r.me == me, r.members == members,
            r.state.term == 0 && r.state.role is Follower && r.state.voted_for.is_none(),
            r.base == 0 && r.base_term == 0 && r.log.len() == 0 && r.snapshot.len() == 0,
            r.commit == 0 && r.applied == 0 && r.leader == me,
            r.reset_timer && r.dirty && !r.pending_snapshot && !r.awaiting_snapshot,
            forall|i: int| 0 <= i < members ==> r.next@[i] == 1 && r.matched@[i] == 0 && !r.votes@[i],
    {
        assert(members > 0 && me < members);
        let mut next: Vec<u64> = Vec::new();
        let mut matched: Vec<u64> = Vec::new();
        let mut votes: Vec<bool> = Vec::new();
        let mut i = 0;
        while i < members
            invariant i <= members, next.len() == i, matched.len() == i, votes.len() == i,
                forall|j: int| 0 <= j < i ==> next@[j] == 1 && matched@[j] == 0 && !votes@[j],
            decreases members - i,
        {
            next.push(1);
            matched.push(0);
            votes.push(false);
            i = i + 1;
        }
        Self {
            state: State::initial(), me, members, base: 0, base_term: 0,
            log: vec![], snapshot: vec![], commit: 0, applied: 0, leader: me,
            reset_timer: true, dirty: true,
            next, matched, votes,
            pending_snapshot: false, awaiting_snapshot: false,
        }
    }
    pub fn durable(&self) -> (d: Durable)
        requires self.wf(),
        ensures d.term == self.state.term, d.voted_for == self.state.voted_for,
            d.base == self.base, d.base_term == self.base_term,
            log::log_view(d.log@) == log::log_view(self.log@),
    {
        Durable { term: self.state.term, voted_for: self.state.voted_for, base: self.base,
            base_term: self.base_term, log: log::suffix(&self.log, 0) }
    }
    pub fn restore(&mut self, d: Durable, snapshot: Vec<u8>) -> (result: bool)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            result ==> self.state.term == d.term && self.state.voted_for == d.voted_for
                && self.base == d.base && self.base_term == d.base_term
                && log::log_view(self.log@) == log::log_view(d.log@) && self.snapshot@ == snapshot@
                && self.state.role is Follower && self.commit == self.base && self.applied == self.base
                && self.pending_snapshot == (self.base != 0) && self.awaiting_snapshot == (self.base != 0) && !self.dirty,
            !result ==> self.persistent_view() == old(self).persistent_view() && self.dirty == old(self).dirty,
    {
        let invalid_vote = match d.voted_for { Some(id) => id >= self.members, None => false };
        if invalid_vote || d.base_term > d.term
            || d.log.len() as u64 > u64::MAX - d.base
            || (d.base == 0 && (d.base_term != 0 || !snapshot.is_empty()))
            || !log::valid_terms(&d.log, d.base_term, d.term)
        { return false; }
        self.state = State { term: d.term, voted_for: d.voted_for, role: Role::Follower };
        self.base = d.base;
        self.base_term = d.base_term;
        self.log = d.log;
        self.snapshot = snapshot;
        self.commit = self.base;
        self.applied = self.base;
        self.pending_snapshot = self.base != 0;
        self.awaiting_snapshot = self.base != 0;
        self.dirty = false;
        true
    }
    pub fn end(&self) -> (r: u64)
        requires self.wf(), ensures r == self.base as int + self.log.len(),
    { self.base + self.log.len() as u64 }
    pub fn term_at(&self, index: u64) -> (r: Option<u64>)
        requires self.wf(),
        ensures r.is_some() == (self.base <= index <= self.base as int + self.log.len()),
            index == self.base ==> r == Some(self.base_term),
            self.base < index <= self.base as int + self.log.len()
                ==> r == Some(self.log@[(index - self.base - 1) as int].term),
    {
        log::term_at(&self.log, self.base, self.base_term, index)
    }
    fn observe(&mut self, term: u64)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.log@ == old(self).log@, self.snapshot@ == old(self).snapshot@,
            self.commit == old(self).commit, self.applied == old(self).applied,
            self.state.term == if term > old(self).state.term { term } else { old(self).state.term },
            self.state.voted_for == if term > old(self).state.term { None } else { old(self).state.voted_for },
            self.state.role == if term > old(self).state.term { Role::Follower } else { old(self).state.role },
    {
        if term > self.state.term {
            self.state.observe_term(term);
            self.dirty = true;
        }
    }
    pub fn election(&mut self) -> (result: Option<Vote>)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.log@ == old(self).log@, self.snapshot@ == old(self).snapshot@,
            self.commit == old(self).commit, self.applied == old(self).applied,
            result.is_some() ==> self.state.term == old(self).state.term + 1
                && self.state.voted_for == Some(self.me)
                && (self.state.role is Candidate || self.state.role is Leader)
                && result.unwrap().term == self.state.term && result.unwrap().candidate == self.me
                && result.unwrap().last_index == self.base as int + self.log.len()
                && result.unwrap().last_term == if self.log.len() == 0 { self.base_term } else { self.log@.last().term },
    {
        if self.state.is_leader() || !self.state.campaign(self.me) { return None; }
        self.dirty = true;
        self.reset_timer = true;
        quorum::reset_votes(&mut self.votes, self.me);
        let request = Vote { term: self.state.term, candidate: self.me,
            last_index: self.end(), last_term: self.term_at(self.end()).unwrap() };
        self.elect_if_majority();
        Some(request)
    }
    fn elect_if_majority(&mut self)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            self.dirty == old(self).dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.log@ == old(self).log@, self.snapshot@ == old(self).snapshot@,
            self.commit == old(self).commit, self.applied == old(self).applied,
            self.state.term == old(self).state.term, self.state.voted_for == old(self).state.voted_for,
            self.state.role == old(self).state.role || self.state.role is Leader,
    {
        let end = self.end();
        let count = quorum::count_votes(&self.votes);
        if self.state.win(self.me, count, self.members) {
            self.leader = self.me;
            // Exhausted indices stop proposals, but heartbeats remain permitted.
            let next = log::successor(end);
            quorum::fill(&mut self.next, next);
            quorum::fill(&mut self.matched, 0);
            self.matched.set(self.me, end);
        }
    }
    pub fn request_vote(&mut self, v: Vote) -> (result: VoteReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.log@ == old(self).log@, self.snapshot@ == old(self).snapshot@,
            self.commit == old(self).commit, self.applied == old(self).applied,
            result.term == self.state.term, self.state.term >= old(self).state.term,
            result.granted ==> self.state.term == v.term && v.candidate < self.members
                && self.state.voted_for == Some(v.candidate)
                && state::log_fresh_spec(v.last_term, v.last_index,
                    if old(self).log.len() == 0 { old(self).base_term } else { old(self).log@.last().term },
                    (old(self).base + old(self).log.len()) as u64),
    {
        let local_index = self.end();
        let local_term = self.term_at(local_index).unwrap();
        let before = (self.state.term, self.state.voted_for);
        let granted = self.state.vote(v.term, v.candidate, self.members,
            v.last_term, v.last_index, local_term, local_index);
        self.dirty = self.dirty || before.0 != self.state.term || before.1 != self.state.voted_for;
        if granted { self.reset_timer = true; }
        VoteReply { term: self.state.term, granted }
    }
    pub fn vote_reply(&mut self, from: usize, sent: u64, r: VoteReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
    {
        if from >= self.members { return; }
        self.observe(r.term);
        if matches!(self.state.role, Role::Candidate) && self.state.term == sent && r.term == sent && r.granted {
            quorum::record_vote(&mut self.votes, from);
            self.elect_if_majority();
        }
    }
    pub fn propose(&mut self, data: &[u8]) -> (result: Option<(u64, u64)>)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.snapshot@ == old(self).snapshot@, self.applied == old(self).applied,
            self.state == old(self).state, self.commit >= old(self).commit,
            result.is_some() ==> old(self).state.role is Leader
                && result.unwrap().0 == old(self).base as int + old(self).log.len() + 1
                && result.unwrap().1 == old(self).state.term
                && log::log_view(self.log@) == log::log_view(old(self).log@).push((old(self).state.term, data@)),
            result.is_none() ==> self.log@ == old(self).log@,
    {
        if !self.state.is_leader() { return None; }
        let end = self.end();
        if end == u64::MAX { return None; }
        let index = end + 1;
        if !log::append(&mut self.log, self.state.term, data) { return None; }
        self.matched.set(self.me, index);
        self.dirty = true;
        self.advance_commit();
        Some((index, self.state.term))
    }
    pub fn replication(&self, peer: usize) -> (result: Option<Replication>)
        requires self.wf(),
        ensures result.is_some() ==> peer < self.members && peer != self.me && self.state.role is Leader,
            match result {
                Some(Replication::Append(a)) => a.term == self.state.term && a.leader == self.me
                    && self.base <= a.prev <= self.base as int + self.log.len() && a.commit == self.commit
                    && a.prev_term == (if a.prev == self.base { self.base_term } else { self.log@[(a.prev - self.base - 1) as int].term })
                    && log::log_view(a.entries@) == log::log_view(self.log@).skip((a.prev - self.base) as int),
                Some(Replication::Snapshot(s)) => s.term == self.state.term && s.leader == self.me
                    && s.index == self.base && s.last_term == self.base_term && s.data@ == self.snapshot@,
                None => true,
            },
    {
        if !self.state.is_leader() || peer >= self.members || peer == self.me { return None; }
        let next = self.next[peer];
        if next <= self.base {
            Some(Replication::Snapshot(Install { term: self.state.term, leader: self.me,
                index: self.base, last_term: self.base_term, data: self.snapshot.clone() }))
        } else {
            let prev = next - 1;
            if prev > self.end() { return None; }
            let offset = (prev - self.base) as usize;
            Some(Replication::Append(Append { term: self.state.term, leader: self.me,
                prev, prev_term: self.term_at(prev).unwrap(), entries: log::suffix(&self.log, offset), commit: self.commit }))
        }
    }
    pub fn append(&mut self, a: Append) -> (result: AppendReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.snapshot@ == old(self).snapshot@, self.applied == old(self).applied,
            self.commit >= old(self).commit, self.state.term >= old(self).state.term,
            result.term == self.state.term,
            result.success ==> self.state.role is Follower,
            log::log_view(self.log@).take((old(self).commit - old(self).base) as int)
                == log::log_view(old(self).log@).take((old(self).commit - old(self).base) as int),
    {
        let mut fail = AppendReply { term: self.state.term, success: false, next: log::successor(self.end()) };
        if a.leader >= self.members { return fail; }
        self.observe(a.term);
        fail.term = self.state.term;
        if a.term != self.state.term { return fail; }
        if !self.state.accept_leader(a.term, a.leader, self.members) { return fail; }
        self.leader = a.leader;
        self.reset_timer = true;
        // All physical repair and follower commit decisions execute the same
        // batch function checked by proof/progress.rs.
        if !a.entries.is_empty() { self.dirty = true; }
        let result = log::merge(&mut self.log, self.base, self.base_term, self.commit,
            a.prev, a.prev_term, &a.entries, a.commit, a.term);
        self.commit = result.commit;
        AppendReply { term: self.state.term, success: result.success, next: result.next }
    }
    pub fn append_reply(&mut self, peer: usize, sent: u64, prev: u64, end: u64, r: AppendReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
    {
        if peer >= self.members || peer == self.me { return; }
        self.observe(r.term);
        if !state::same_epoch(self.state.role, self.state.term, sent) || r.term != sent { return; }
        if r.success {
            let matched = self.matched[peer].max(end.min(self.end()));
            self.matched.set(peer, matched);
            let next = log::successor(self.matched[peer]);
            self.next.set(peer, next);
            self.advance_commit();
        } else if self.next[peer] == log::successor(prev) {
            // Failed obsolete RPCs cannot undo newer successful acknowledgments.
            let next = r.next.min(prev).min(log::successor(self.end())).max(log::successor(self.matched[peer])).max(1);
            self.next.set(peer, next);
        }
    }
    fn advance_commit(&mut self)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            self.dirty == old(self).dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.base == old(self).base, self.base_term == old(self).base_term,
            self.log@ == old(self).log@, self.snapshot@ == old(self).snapshot@,
            self.applied == old(self).applied, self.state == old(self).state,
            self.commit >= old(self).commit,
    {
        if self.state.is_leader() {
            self.commit = quorum::select_commit(&self.log, self.base, self.commit, self.state.term,
                &self.matched, self.members);
        }
    }
    pub fn compact(&mut self, index: u64, data: &[u8]) -> (result: bool)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            self.state == old(self).state, self.commit == old(self).commit, self.applied == old(self).applied,
            self.base as int + self.log.len() == old(self).base as int + old(self).log.len(),
            result ==> self.base == index && log::log_view(self.log@) == log::log_view(old(self).log@).skip((index - old(self).base) as int),
            !result ==> self.base == old(self).base && self.log@ == old(self).log@,
    {
        if !cursor::compact(&mut self.log, &mut self.base, &mut self.base_term,
            self.applied, self.commit, index) { return false; }
        self.snapshot = log::copy_bytes(data);
        self.dirty = true;
        true
    }
    pub fn install(&mut self, s: Install) -> (result: InstallReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
    {
        if s.leader >= self.members { return InstallReply { term: self.state.term, accepted: false }; }
        self.observe(s.term);
        if s.term != self.state.term || s.last_term > s.term { return InstallReply { term: self.state.term, accepted: false }; }
        if !self.state.accept_leader(s.term, s.leader, self.members) { return InstallReply { term: self.state.term, accepted: false }; }
        self.leader = s.leader;
        self.reset_timer = true;
        if cursor::install(&mut self.log, &mut self.base, &mut self.base_term,
            &mut self.commit, &mut self.applied, s.index, s.last_term) {
            self.snapshot = s.data;
            self.pending_snapshot = true;
            self.awaiting_snapshot = true;
            self.dirty = true;
        }
        InstallReply { term: self.state.term, accepted: true }
    }
    pub fn install_reply(&mut self, peer: usize, sent: u64, index: u64, r: InstallReply)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
    {
        if peer >= self.members || peer == self.me { return; }
        self.observe(r.term);
        if state::same_epoch(self.state.role, self.state.term, sent) && r.term == sent && r.accepted {
            let matched = self.matched[peer].max(index.min(self.end()));
            self.matched.set(peer, matched);
            let next = log::successor(self.matched[peer]);
            self.next.set(peer, next);
            self.advance_commit();
        }
    }
    pub fn take_applied(&mut self) -> (result: Vec<Applied>)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            old(self).dirty ==> self.dirty,
            !self.dirty ==> self.persistent_view() == old(self).persistent_view(),
            old(self).awaiting_snapshot ==> result.len() == (if old(self).pending_snapshot { 1int } else { 0int }),
            old(self).awaiting_snapshot ==> forall|i: int| 0 <= i < result.len() ==> result@[i] is Snapshot,
    {
        let mut result = vec![];
        if self.pending_snapshot {
            result.push(Applied::Snapshot { index: self.base, term: self.base_term, data: self.snapshot.clone() });
            self.pending_snapshot = false;
        }
        if self.awaiting_snapshot { return result; }
        while self.applied < self.commit
            invariant self.wf(), self.me == old(self).me, self.members == old(self).members,
                self.commit == old(self).commit, self.dirty == old(self).dirty,
                self.persistent_view() == old(self).persistent_view(),
            decreases self.commit - self.applied,
        {
            let command = cursor::next_application(&self.log, self.base, self.commit, &mut self.applied).unwrap();
            result.push(Applied::Command { index: command.index, data: command.entry.data });
        }
        result
    }
    pub fn acknowledge_snapshot(&mut self, term: u64, index: u64, data: &[u8]) -> (accepted: bool)
        requires old(self).wf(),
        ensures self.wf(), self.me == old(self).me, self.members == old(self).members,
            self.dirty == old(self).dirty, self.persistent_view() == old(self).persistent_view(),
            accepted ==> old(self).awaiting_snapshot && !old(self).pending_snapshot
                && index == old(self).base && term == old(self).base_term && data@ == old(self).snapshot@,
            self.awaiting_snapshot == (old(self).awaiting_snapshot && !accepted),
    {
        if !self.awaiting_snapshot || self.pending_snapshot || !self.snapshot_is_current(term, index, data) { return false; }
        self.awaiting_snapshot = false;
        true
    }

    pub fn snapshot_is_current(&self, term: u64, index: u64, data: &[u8]) -> (r: bool)
        ensures r == (index == self.base && term == self.base_term && data@ == self.snapshot@),
    {
        index == self.base && term == self.base_term && log::bytes_equal(data, &self.snapshot)
    }
}

} // verus!

#[cfg(test)]
mod regressions {
    use super::*;
    #[test]
    fn obsolete_success_and_failure_cannot_rewrite_replication_progress() {
        let mut c = Core::new(0, 3);
        let vote = c.election().unwrap();
        c.vote_reply(1, vote.term, VoteReply { term: vote.term, granted: true });
        let (index, term) = c.propose(b"command").unwrap();
        c.append_reply(1, term, 0, index, AppendReply { term, success: true, next: 2 });
        c.append_reply(1, term, 0, 0, AppendReply { term, success: false, next: 1 });
        assert_eq!(c.matched[1], 1);
        assert_eq!(c.next[1], 2);
        c.state.observe_term(term + 1);
        c.append_reply(2, term, 0, index, AppendReply { term, success: true, next: 2 });
        assert_eq!(c.matched[2], 0);
    }
    #[test]
    fn old_term_majority_does_not_authorize_commit() {
        let mut c = Core::new(0, 3);
        c.state = State { term: 4, role: Role::Leader, voted_for: Some(0) };
        c.log.push(Entry { term: 3, data: b"old".to_vec() });
        c.matched.fill(1);
        c.votes[0] = true;
        c.votes[1] = true;
        c.advance_commit();
        assert_eq!(c.commit, 0);
        c.propose(b"current").unwrap();
        c.append_reply(1, 4, 1, 2, AppendReply { term: 4, success: true, next: 3 });
        assert_eq!(c.commit, 2);
        let applied = c.take_applied();
        assert!(matches!(&applied[..], [Applied::Command { index: 1, .. }, Applied::Command { index: 2, .. }]));
    }
    #[test]
    fn application_waits_for_installed_snapshot_acknowledgment() {
        let mut c = Core::new(0, 3);
        c.install(Install { term: 4, leader: 1, index: 10, last_term: 3, data: b"snapshot A".to_vec() });
        assert!(matches!(&c.take_applied()[..], [Applied::Snapshot { index: 10, .. }]));
        let reply = c.append(Append { term: 4, leader: 1, prev: 10, prev_term: 3,
            entries: vec![Entry { term: 4, data: b"command 11".to_vec() }], commit: 11 });
        assert!(reply.success);
        assert!(c.take_applied().is_empty(), "suffix delivery must wait for the service snapshot acknowledgment");
        c.install(Install { term: 4, leader: 1, index: 20, last_term: 4, data: b"snapshot B".to_vec() });
        assert!(matches!(&c.take_applied()[..], [Applied::Snapshot { index: 20, .. }]));
        assert!(!c.acknowledge_snapshot(3, 10, b"snapshot A"));
        c.append(Append { term: 4, leader: 1, prev: 20, prev_term: 4,
            entries: vec![Entry { term: 4, data: b"command 21".to_vec() }], commit: 21 });
        assert!(c.take_applied().is_empty());
        assert!(c.acknowledge_snapshot(4, 20, b"snapshot B"));
        assert!(matches!(&c.take_applied()[..], [Applied::Command { index: 21, .. }]));
    }

    #[test]
    fn exhausted_terms_and_indices_reject_new_work() {
        let mut c = Core::new(0, 1);
        c.state.term = u64::MAX;
        assert!(c.election().is_none());
        assert_eq!(c.state.role, Role::Follower);
        c.state.role = Role::Leader;
        c.base = u64::MAX;
        c.base_term = u64::MAX;
        c.commit = u64::MAX;
        c.applied = u64::MAX;
        c.state.voted_for = Some(0);
        c.votes[0] = true;
        c.next[0] = u64::MAX;
        c.matched[0] = u64::MAX;
        assert!(c.propose(b"overflow").is_none());
        assert!(c.take_applied().is_empty());
    }
}
