//! Verified request/storage/reply phases used by the asynchronous Raft host.
use super::state::{self, State, VoteRequest, VoteReply};
use super::election;
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(verus_keep_ghost), derive(serde::Serialize, serde::Deserialize))]
pub struct VoteImage {
    pub version: u64,
    pub term: u64,
    pub voted_for: Option<usize>,
    pub me: usize,
    pub members: usize,
}

// Only checked vote/term preparation constructs this operation. The host borrows its image while
// writing, then returns the owned operation after successful I/O completion.
pub struct WriteVote { id: u64, image: VoteImage }

#[derive(Clone, Copy)]
struct PendingVote {
    id: u64,
    image: VoteImage,
    request: VoteRequest,
    reply: VoteReply,
    before: Ghost<State>,
}

pub struct Machine {
    state: State,
    serial: u64,
    pending: Option<PendingVote>,
}

pub open spec fn image(s: State) -> VoteImage {
    VoteImage { version: 1, term: s.term, voted_for: s.voted_for, me: s.me, members: s.members }
}

// Observable local operation state, including the pending request and exact reply.
pub struct OperationView {
    pub state: State,
    pub serial: u64,
    pub pending: Option<(u64, VoteImage, VoteRequest, VoteReply, State)>,
}

pub open spec fn view_invariant(v: OperationView) -> bool {
    state::well_formed(v.state) && v.state.last_index == 0 && v.state.last_term == 0
    && match v.pending {
        None => true,
        Some(p) => p.0 == v.serial && p.1 == image(v.state)
            && state::well_formed(p.4) && state::vote_step(p.4, p.2, v.state, p.3),
    }
}

pub open spec fn initial_view(v: OperationView, me: usize, members: usize) -> bool {
    state::initial(v.state, me, members) && v.serial == 0 && v.pending.is_none()
}

pub open spec fn begin_step(s: OperationView, q: VoteRequest, t: OperationView,
    output: Option<(u64, VoteImage)>) -> bool {
    if s.pending.is_some() || s.serial == u64::MAX {
        t == s && output.is_none()
    } else {
        t.serial == s.serial + 1 && t.pending.is_some()
        && t.pending.unwrap().0 == t.serial && t.pending.unwrap().1 == image(t.state)
        && t.pending.unwrap().2 == q && t.pending.unwrap().4 == s.state
        && state::vote_step(s.state, q, t.state, t.pending.unwrap().3)
        && output == Some((t.serial, image(t.state)))
    }
}

pub open spec fn complete_step(s: OperationView, id: u64, saved: VoteImage,
    t: OperationView, reply: Option<VoteReply>) -> bool {
    if s.pending.is_some() && s.pending.unwrap().0 == id && s.pending.unwrap().1 == saved {
        t.state == s.state && t.serial == s.serial && t.pending.is_none()
        && reply == Some(s.pending.unwrap().3)
    } else { t == s && reply.is_none() }
}

pub open spec fn campaign_step(s: OperationView, term: u64, t: OperationView, ok: bool) -> bool {
    ok == (s.pending.is_none() && s.state.term == term && s.state.voted_for == Some(s.state.me))
    && t.serial == s.serial && t.pending == s.pending
    && t.state == state::State { role: if ok { state::Role::Candidate } else { s.state.role }, ..s.state }
}

pub open spec fn elect_step(s: OperationView, votes: election::TallyView, t: OperationView, ok: bool) -> bool {
    election::valid(votes)
    && ok == (s.pending.is_none() && s.state.role == state::Role::Candidate
        && s.state.term == votes.request.term && s.state.me == votes.request.candidate
        && s.state.members == votes.members && s.state.voted_for == Some(s.state.me)
        && votes.voters.len() > votes.members / 2)
    && t.serial == s.serial && t.pending == s.pending
    && t.state == State { role: if ok { state::Role::Leader } else { s.state.role }, ..s.state }
}

pub struct TermContext { id: u64, previous: VoteImage, peer: usize, reply: VoteReply }
pub struct TermView { pub id: u64, pub previous: VoteImage, pub peer: usize, pub reply: VoteReply }
pub open spec fn term_image(c: TermView) -> VoteImage {
    VoteImage { term: c.reply.term, voted_for: None, ..c.previous }
}
pub open spec fn term_begin(s: OperationView, peer: usize, reply: VoteReply, t: OperationView, c: Option<TermView>) -> bool {
    if s.pending.is_none() && s.serial < u64::MAX && peer < s.state.members && reply.term > s.state.term {
        t.state == State { term: reply.term, voted_for: None, role: state::Role::Follower, ..s.state }
        && t.pending == s.pending && t.serial == s.serial + 1
        && c == Some(TermView { id: t.serial, previous: image(s.state), peer, reply })
    } else { t == s && c.is_none() }
}
pub open spec fn term_end(s: OperationView, c: TermView, id: u64, saved: VoteImage, t: OperationView, ok: bool) -> bool {
    ok == (s.pending.is_none() && s.serial == c.id && id == c.id && saved == term_image(c)
        && image(s.state) == term_image(c) && c.peer < s.state.members && c.reply.term > c.previous.term)
    && t == s
}
impl TermContext {
    pub closed spec fn view(&self) -> TermView { TermView { id: self.id, previous: self.previous, peer: self.peer, reply: self.reply } }
}

impl WriteVote {
    pub closed spec fn number(&self) -> u64 { self.id }
    pub closed spec fn contents(&self) -> VoteImage { self.image }
    pub fn sequence(&self) -> (r: u64) ensures r == self.number(), { self.id }
    pub fn image(&self) -> (r: VoteImage) ensures r == self.contents(), { self.image }
}

impl Machine {
    pub closed spec fn view(&self) -> OperationView {
        OperationView { state: self.state, serial: self.serial,
            pending: match self.pending { None => None,
                Some(p) => Some((p.id, p.image, p.request, p.reply, p.before@)) } }
    }
    pub proof fn view_is_valid(&self)
        requires self.invariant(),
        ensures view_invariant(self.view()), self.view().state == self.state(),
    {}
    pub closed spec fn state(&self) -> State { self.state }
    pub closed spec fn idle(&self) -> bool { self.pending.is_none() }
    pub closed spec fn invariant(&self) -> bool {
        state::well_formed(self.state) && self.state.last_index == 0 && self.state.last_term == 0
        && match self.pending {
            None => true,
            Some(p) => p.id == self.serial && p.image == image(self.state)
                && state::well_formed(p.before@)
                && state::vote_step(p.before@, p.request, self.state, p.reply),
        }
    }
    pub closed spec fn pending_request(&self) -> Option<VoteRequest> {
        match self.pending { None => None, Some(p) => Some(p.request) }
    }
    pub closed spec fn pending_before(&self) -> State {
        self.pending.unwrap().before@
    }
    pub closed spec fn accepts(&self, op: &WriteVote) -> bool {
        match self.pending { None => false, Some(p) => p.id == op.id && p.image == op.image }
    }

    pub proof fn matching_operation(&self, op: &WriteVote)
        requires self.invariant(), self.accepts(op),
        ensures op.contents() == image(self.state()),
    {}

    pub fn new(me: usize, members: usize) -> (r: Option<Self>)
        ensures r.is_some() == (members > 0 && me < members),
            r.is_some() ==> r.unwrap().invariant() && r.unwrap().idle()
                && state::initial(r.unwrap().state(), me, members)
                && initial_view(r.unwrap().view(), me, members),
    {
        match State::new(me, members) {
            None => None,
            Some(s) => Some(Self { state: s, serial: 0, pending: None }),
        }
    }

    // Version and deployment identity are checked inside verified code.
    pub fn recover(me: usize, members: usize, saved: VoteImage) -> (r: Option<Self>)
        ensures r.is_some() ==> r.unwrap().invariant() && r.unwrap().idle()
            && r.unwrap().state().me == me && r.unwrap().state().members == members
            && r.unwrap().state().term == saved.term
            && r.unwrap().state().voted_for == saved.voted_for
            && image(r.unwrap().state()) == saved
            && r.unwrap().state().role == state::Role::Follower
            && r.unwrap().view().pending.is_none() && r.unwrap().view().serial == 0,
    {
        if saved.version != 1 || saved.me != me || saved.members != members { return None; }
        match State::restore_vote(me, members, saved.term, saved.voted_for) {
            None => None,
            Some(s) => Some(Self { state: s, serial: 0, pending: None }),
        }
    }

    pub fn term(&self) -> (r: u64) ensures r == self.state().term, { self.state.term }
    pub fn is_leader(&self) -> (r: bool)
        ensures r == (self.state().role == state::Role::Leader),
    { self.state.is_leader() }

    pub fn election_request(&self) -> (r: Option<state::VoteRequest>)
        ensures r.is_some() == (self.state().term < u64::MAX),
            r.is_some() ==> r.unwrap().term == self.view().state.term + 1
                && r.unwrap().candidate == self.view().state.me
                && r.unwrap().last_index == self.view().state.last_index
                && r.unwrap().last_term == self.view().state.last_term,
    { self.state.election_request() }

    pub fn campaign(&mut self, term: u64) -> (r: bool)
        requires old(self).invariant(),
        ensures self.invariant(), campaign_step(old(self).view(), term, self.view(), r),
    {
        if self.pending.is_some() { return false; }
        self.state.campaign(term)
    }

    pub fn elect(&mut self, votes: &election::Tally) -> (r: bool)
        requires old(self).invariant(), votes.invariant(),
        ensures self.invariant(), elect_step(old(self).view(), votes.view(), self.view(), r),
    {
        proof { votes.evidence(); }
        let q = votes.request();
        let ok = self.pending.is_none() && matches!(self.state.role, state::Role::Candidate)
            && self.state.term == q.term && self.state.me == q.candidate
            && self.state.members == votes.members() && self.state.voted_for == Some(self.state.me)
            && votes.has_quorum();
        if ok { self.state.role = state::Role::Leader; }
        ok
    }

    pub fn prepare_term(&mut self, peer: usize, reply: VoteReply) -> (r: Option<(WriteVote, TermContext)>)
        requires old(self).invariant(),
        ensures self.invariant(), term_begin(old(self).view(), peer, reply, self.view(),
            match r { None => None, Some((_, c)) => Some(c.view()) }),
            r.is_some() ==> r.unwrap().0.number() == r.unwrap().1.view().id
                && r.unwrap().0.contents() == term_image(r.unwrap().1.view()),
    {
        if self.pending.is_some() || self.serial == u64::MAX || peer >= self.state.members || reply.term <= self.state.term { return None; }
        self.serial = self.serial + 1;
        let previous = VoteImage { version: 1, term: self.state.term, voted_for: self.state.voted_for,
            me: self.state.me, members: self.state.members };
        let saved = VoteImage { term: reply.term, voted_for: None, ..previous };
        self.state.term = reply.term;
        self.state.voted_for = None;
        self.state.role = state::Role::Follower;
        Some((WriteVote { id: self.serial, image: saved }, TermContext { id: self.serial, previous, peer, reply }))
    }
    pub fn complete_term(&mut self, c: TermContext, op: WriteVote) -> (ok: bool)
        requires old(self).invariant(),
        ensures self.invariant(), term_end(old(self).view(), c.view(), op.number(), op.contents(), self.view(), ok),
    {
        let yes = self.pending.is_none() && self.serial == c.id && op.id == c.id
            && op.image.version == c.previous.version && op.image.term == c.reply.term
            && op.image.voted_for.is_none() && op.image.me == c.previous.me && op.image.members == c.previous.members
            && c.previous.version == 1 && c.reply.term == self.state.term && self.state.voted_for.is_none()
            && c.previous.me == self.state.me && c.previous.members == self.state.members
            && c.peer < self.state.members && c.reply.term > c.previous.term;
        yes
    }

    pub fn pending_matches(&self, request: state::VoteRequest) -> (r: bool)
        ensures r == (self.pending_request() == Some(request)),
            r ==> self.view().pending.is_some() && self.view().pending.unwrap().2 == request,
    {
        match self.pending {
            None => false,
            Some(p) => p.request.term == request.term && p.request.candidate == request.candidate
                && p.request.last_index == request.last_index && p.request.last_term == request.last_term,
        }
    }

    pub fn begin_vote(&mut self, request: VoteRequest) -> (op: Option<WriteVote>)
        requires old(self).invariant(),
        ensures self.invariant(),
            op.is_some() ==> self.accepts(&op.unwrap())
                && op.unwrap().contents() == image(self.state())
                && self.pending_request() == Some(request)
                && self.pending_before() == old(self).state(),
            op.is_none() ==> self.state() == old(self).state(),
            begin_step(old(self).view(), request, self.view(),
                match op { None => None, Some(o) => Some((o.number(), o.contents())) }),
    {
        if self.pending.is_some() || self.serial == u64::MAX { return None; }
        let ghost before = self.state;
        let reply = self.state.request_vote(request);
        self.serial = self.serial + 1;
        let saved = VoteImage { version: 1, term: self.state.term,
            voted_for: self.state.voted_for, me: self.state.me, members: self.state.members };
        self.pending = Some(PendingVote { id: self.serial, image: saved,
            request, reply, before: Ghost(before) });
        Some(WriteVote { id: self.serial, image: saved })
    }

    // This is reply authorization, not a claim that arbitrary input is durable.
    // The host caller must supply this operation only after its real write returns.
    pub fn complete_vote(&mut self, completed: WriteVote) -> (reply: Option<VoteReply>)
        requires old(self).invariant(),
        ensures self.invariant(), self.state() == old(self).state(),
            reply.is_some() == old(self).accepts(&completed),
            reply.is_some() ==> self.idle()
                && state::vote_step(old(self).pending_before(),
                    old(self).pending_request().unwrap(), self.state(), reply.unwrap())
                && completed.contents() == image(self.state()),
            complete_step(old(self).view(), completed.number(), completed.contents(), self.view(), reply),
    {
        match self.pending {
            None => None,
            Some(p) => {
                let same = p.id == completed.id
                    && p.image.version == completed.image.version
                    && p.image.term == completed.image.term
                    && p.image.voted_for == completed.image.voted_for
                    && p.image.me == completed.image.me
                    && p.image.members == completed.image.members;
                if !same { return None; }
                self.pending = None;
                Some(p.reply)
            }
        }
    }
}
}
