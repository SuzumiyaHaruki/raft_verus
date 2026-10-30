//! Executable vote-call contexts and distinct reply collection.
//! RPC transport provenance and durable self-vote composition remain caller obligations.
use super::state::{VoteRequest, VoteReply};
use vstd::prelude::*;
verus! {
pub struct VoteCall { request: VoteRequest, peer: usize }
pub struct Tally {
    request: VoteRequest,
    members: usize,
    voters: Vec<usize>,
    replies: Ghost<Seq<(usize, Option<VoteReply>)>>,
}
pub struct TallyView {
    pub request: VoteRequest,
    pub members: usize,
    pub voters: Seq<usize>,
    pub replies: Seq<(usize, Option<VoteReply>)>,
}
pub open spec fn granted(v: TallyView, peer: usize, reply: Option<VoteReply>) -> bool {
    peer < v.members && reply.is_some() && reply.unwrap().term == v.request.term && reply.unwrap().granted
}
pub open spec fn valid(v: TallyView) -> bool {
    v.members > 0 && v.request.candidate < v.members
    && v.voters.no_duplicates()
    && forall|i: int| 0 <= i < v.voters.len() ==> #[trigger] v.voters[i] < v.members
        && exists|j: int| 0 <= j < v.replies.len() && v.replies[j].0 == v.voters[i]
            && granted(v, v.voters[i], v.replies[j].1)
}
pub open spec fn record_step(s: TallyView, peer: usize, reply: Option<VoteReply>, t: TallyView) -> bool {
    t.request == s.request && t.members == s.members
    && t.replies == s.replies.push((peer, reply))
    && t.voters == if granted(s, peer, reply) && !s.voters.contains(peer) { s.voters.push(peer) } else { s.voters }
}
impl VoteCall {
    pub closed spec fn request(&self) -> VoteRequest { self.request }
    pub closed spec fn peer(&self) -> usize { self.peer }
    pub fn args(&self) -> (q: VoteRequest) ensures q == self.request(), { self.request }
    pub fn destination(&self) -> (p: usize) ensures p == self.peer(), { self.peer }
}
impl Tally {
    pub closed spec fn view(&self) -> TallyView {
        TallyView { request: self.request, members: self.members, voters: self.voters@, replies: self.replies@ }
    }
    pub closed spec fn invariant(&self) -> bool { valid(self.view()) }
    pub fn new(request: VoteRequest, members: usize) -> (r: Option<Self>)
        ensures r.is_some() == (members > 0 && request.candidate < members),
            r.is_some() ==> r.unwrap().invariant() && r.unwrap().view().request == request
                && r.unwrap().view().members == members && r.unwrap().view().voters.len() == 0
                && r.unwrap().view().replies.len() == 0,
    {
        if members == 0 || request.candidate >= members { return None; }
        Some(Self { request, members, voters: Vec::new(), replies: Ghost(Seq::empty()) })
    }
    pub fn outbound(&self, peer: usize) -> (r: Option<VoteCall>)
        ensures r.is_some() == (peer < self.view().members && peer != self.view().request.candidate),
            r.is_some() ==> r.unwrap().request() == self.view().request && r.unwrap().peer() == peer,
    {
        if peer >= self.members || peer == self.request.candidate { return None; }
        Some(VoteCall { request: self.request, peer })
    }
    fn contains(&self, peer: usize) -> (r: bool)
        ensures r == self.view().voters.contains(peer),
    {
        let mut i = 0;
        while i < self.voters.len()
            invariant i <= self.voters.len(),
                forall|j: int| 0 <= j < i ==> self.voters[j] != peer,
            decreases self.voters.len() - i,
        {
            if self.voters[i] == peer { return true; }
            i += 1;
        }
        false
    }
    pub proof fn evidence(&self)
        requires self.invariant(),
        ensures valid(self.view()),
    {}
    pub fn record(&mut self, peer: usize, reply: Option<VoteReply>)
        requires old(self).invariant(),
        ensures self.invariant(), record_step(old(self).view(), peer, reply, self.view()),
    {
        let ghost before = self.view();
        self.replies = Ghost(self.replies@.push((peer, reply)));
        let accepted = match reply { Some(r) => peer < self.members && r.term == self.request.term && r.granted, None => false };
        if accepted && !self.contains(peer) {
            self.voters.push(peer);
            assert(self.voters@.no_duplicates());
        }
        let ghost after = self.view();
        assert forall|i: int| 0 <= i < after.voters.len() implies #[trigger] after.voters[i] < after.members
            && exists|j: int| 0 <= j < after.replies.len() && after.replies[j].0 == after.voters[i]
                && granted(after, after.voters[i], after.replies[j].1) by {
            if i < before.voters.len() {
                assert(before.voters[i] < before.members);
                let j = choose|j: int| 0 <= j < before.replies.len() && before.replies[j].0 == before.voters[i]
                    && granted(before, before.voters[i], before.replies[j].1);
                assert(after.replies[j] == before.replies[j]);
                assert(granted(after, after.voters[i], after.replies[j].1));
            } else {
                let j = before.replies.len() as int;
                assert(after.replies[j] == (peer, reply));
                assert(granted(after, after.voters[i], after.replies[j].1));
            }
        }
    }
    pub fn complete_call(&mut self, call: VoteCall, reply: Option<VoteReply>)
        requires old(self).invariant(),
        ensures self.invariant(),
            call.request() == old(self).view().request ==>
                record_step(old(self).view(), call.peer(), reply, self.view()),
            call.request() != old(self).view().request ==> self.view() == old(self).view(),
    {
        let q = call.request;
        if q.term != self.request.term || q.candidate != self.request.candidate
            || q.last_index != self.request.last_index || q.last_term != self.request.last_term { return; }
        self.record(call.peer, reply);
    }
    pub fn request(&self) -> (r: VoteRequest) ensures r == self.view().request, { self.request }
    pub fn members(&self) -> (r: usize) ensures r == self.view().members, { self.members }
    pub fn has_quorum(&self) -> (r: bool)
        requires self.invariant(),
        ensures r == (self.view().voters.len() > self.view().members / 2),
    { self.voters.len() > self.members / 2 }
}
}
