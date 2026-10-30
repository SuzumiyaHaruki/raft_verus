//! Per-incarnation execution journal of the real vote runtime.
//! Ghost history is updated by actual constructors and handlers; it is not a
//! replacement for cluster provenance or durable cross-crash history.
use super::{protocol as p, state, storage, election};
use vstd::prelude::*;
verus! {
pub enum Origin { Fresh(usize, usize), Recovered(p::VoteImage) }
pub enum Input { Request(state::VoteRequest), SelfVote(state::VoteRequest), Campaign(u64), Elect(election::TallyView), Release(u64, p::VoteImage) }
pub enum Output { Submission(Option<(u64, p::VoteImage)>), Campaign(bool), Elected(bool), Reply(Option<state::VoteReply>) }
pub struct History {
    pub origin: Origin,
    pub states: Seq<p::OperationView>,
    pub inputs: Seq<Input>,
    pub outputs: Seq<Output>,
}
pub open spec fn start(v: p::OperationView, origin: Origin) -> bool {
    p::view_invariant(v) && match origin {
        Origin::Fresh(me, members) => p::initial_view(v, me, members),
        Origin::Recovered(image) => p::image(v.state) == image && v.state.role == state::Role::Follower
            && v.pending.is_none() && v.serial == 0,
    }
}
pub open spec fn step(s: p::OperationView, i: Input, t: p::OperationView, o: Output) -> bool {
    match (i, o) {
        (Input::Request(q), Output::Submission(op)) => p::begin_step(s, q, t, op),
        (Input::SelfVote(q), Output::Submission(op)) =>
            q.term == s.state.term + 1 && q.candidate == s.state.me
            && q.last_index == s.state.last_index && q.last_term == s.state.last_term
            && p::begin_step(s, q, t, op),
        (Input::Campaign(term), Output::Campaign(ok)) => p::campaign_step(s, term, t, ok),
        (Input::Elect(v), Output::Elected(ok)) => p::elect_step(s, v, t, ok),
        (Input::Release(id, image), Output::Reply(reply)) => p::complete_step(s, id, image, t, reply),
        _ => false,
    }
}
pub open spec fn valid(h: History) -> bool {
    h.states.len() == h.inputs.len() + 1 && h.outputs.len() == h.inputs.len()
    && start(h.states[0], h.origin)
    && forall|k: int| 0 <= k < h.inputs.len() ==>
        step(h.states[k], #[trigger] h.inputs[k], h.states[k+1], h.outputs[k])
}
pub proof fn extend(h: History, i: Input, t: p::OperationView, o: Output) -> (next: History)
    requires valid(h), step(h.states.last(), i, t, o),
    ensures valid(next), next.states.last() == t,
        next.origin == h.origin, next.states == h.states.push(t),
        next.inputs == h.inputs.push(i), next.outputs == h.outputs.push(o),
{
    let next = History { origin: h.origin, states: h.states.push(t),
        inputs: h.inputs.push(i), outputs: h.outputs.push(o) };
    assert forall|k: int| 0 <= k < next.inputs.len() implies
        step(next.states[k], #[trigger] next.inputs[k], next.states[k+1], next.outputs[k]) by {
        if k < h.inputs.len() {
            assert(step(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
        } else { assert(k == h.inputs.len()); }
    }
    next
}

// The async RPC future carries this immutable caller context across storage.
// Fields are private, so the host cannot retarget a completion to a new request.
pub struct ReplyContext {
    id: u64,
    image: p::VoteImage,
    request: state::VoteRequest,
    before: Ghost<p::OperationView>,
}
impl ReplyContext {
    pub closed spec fn id(&self) -> u64 { self.id }
    pub closed spec fn image(&self) -> p::VoteImage { self.image }
    pub closed spec fn request(&self) -> state::VoteRequest { self.request }
    pub closed spec fn before(&self) -> p::OperationView { self.before@ }
}

pub struct Runtime {
    core: p::Machine,
    journal: Ghost<History>,
}
impl Runtime {
    pub closed spec fn history(&self) -> History { self.journal@ }
    pub closed spec fn view(&self) -> p::OperationView { self.core.view() }
    pub closed spec fn invariant(&self) -> bool {
        self.core.invariant() && valid(self.journal@)
        && self.journal@.states.last() == self.core.view()
    }
    pub proof fn connected(&self)
        requires self.invariant(),
        ensures valid(self.history()), self.history().states.last() == self.view(),
            p::view_invariant(self.view()),
    { self.core.view_is_valid(); }

    pub fn new(me: usize, members: usize) -> (r: Option<Self>)
        ensures r.is_some() == (members > 0 && me < members),
            r.is_some() ==> r.unwrap().invariant()
                && r.unwrap().history().inputs.len() == 0
                && r.unwrap().history().origin == Origin::Fresh(me, members),
    {
        match p::Machine::new(me, members) {
            None => None,
            Some(core) => {
                proof { core.view_is_valid(); }
                let journal = Ghost(History { origin: Origin::Fresh(me, members),
                    states: seq![core.view()], inputs: Seq::empty(), outputs: Seq::empty() });
                Some(Self { core, journal })
            }
        }
    }
    pub fn recover(me: usize, members: usize, image: p::VoteImage) -> (r: Option<Self>)
        ensures r.is_some() ==> r.unwrap().invariant()
            && r.unwrap().history().origin == Origin::Recovered(image)
            && r.unwrap().history().inputs.len() == 0,
    {
        match p::Machine::recover(me, members, image) {
            None => None,
            Some(core) => {
                proof { core.view_is_valid(); }
                let journal = Ghost(History { origin: Origin::Recovered(image),
                    states: seq![core.view()], inputs: Seq::empty(), outputs: Seq::empty() });
                Some(Self { core, journal })
            }
        }
    }
    pub fn term(&self) -> (r: u64)
        requires self.invariant(),
        ensures r == self.view().state.term,
    { proof { self.core.view_is_valid(); } self.core.term() }
    pub fn is_leader(&self) -> (r: bool)
        requires self.invariant(),
        ensures r == (self.view().state.role == state::Role::Leader),
    { proof { self.core.view_is_valid(); } self.core.is_leader() }

    pub fn begin_vote(&mut self, q: state::VoteRequest) -> (op: Option<p::WriteVote>)
        requires old(self).invariant(),
        ensures self.invariant(),
            self.history().inputs == old(self).history().inputs.push(Input::Request(q)),
            self.history().states == old(self).history().states.push(self.view()),
            self.history().outputs == old(self).history().outputs.push(Output::Submission(
                match op { None => None, Some(w) => Some((w.number(), w.contents())) })),
    {
        let op = self.core.begin_vote(q);
        let ghost output = Output::Submission(match op { None => None,
            Some(ref w) => Some((w.number(), w.contents())) });
        self.journal = Ghost(extend(self.journal@, Input::Request(q), self.core.view(), output));
        op
    }
    pub fn begin_election(&mut self) -> (r: Option<(state::VoteRequest, p::WriteVote, ReplyContext)>)
        requires old(self).invariant(),
        ensures self.invariant(),
            r.is_some() ==> r.unwrap().0.candidate == old(self).view().state.me
                && r.unwrap().0.term == old(self).view().state.term + 1
                && r.unwrap().2.request() == r.unwrap().0,
    {
        let request = match self.core.election_request() { Some(q) => q, None => return None };
        let ghost before = self.core.view();
        let op = self.core.begin_vote(request);
        let ghost output = Output::Submission(match op { None => None,
            Some(ref w) => Some((w.number(), w.contents())) });
        self.journal = Ghost(extend(self.journal@, Input::SelfVote(request), self.core.view(), output));
        match op {
            None => None,
            Some(operation) => {
                let context = ReplyContext { id: operation.sequence(), image: operation.image(),
                    request, before: Ghost(before) };
                Some((request, operation, context))
            }
        }
    }
    pub fn campaign(&mut self, term: u64) -> (r: bool)
        requires old(self).invariant(),
        ensures self.invariant(),
            r ==> self.view().state.term == term && self.view().state.role == state::Role::Candidate
                && self.view().state.voted_for == Some(self.view().state.me),
    {
        let ok = self.core.campaign(term);
        self.journal = Ghost(extend(self.journal@, Input::Campaign(term), self.core.view(), Output::Campaign(ok)));
        ok
    }

    pub fn elect(&mut self, votes: &election::Tally) -> (r: bool)
        requires old(self).invariant(), votes.invariant(),
        ensures self.invariant(), p::elect_step(old(self).view(), votes.view(), self.view(), r),
            self.history().inputs == old(self).history().inputs.push(Input::Elect(votes.view())),
    {
        let ok = self.core.elect(votes);
        self.journal = Ghost(extend(self.journal@, Input::Elect(votes.view()), self.core.view(), Output::Elected(ok)));
        ok
    }

    pub fn begin_rpc(&mut self, request: state::VoteRequest)
        -> (r: Option<(p::WriteVote, ReplyContext)>)
        requires old(self).invariant(),
        ensures self.invariant(),
            r.is_some() ==> r.unwrap().1.id() == r.unwrap().0.number()
                && r.unwrap().1.image() == r.unwrap().0.contents()
                && r.unwrap().1.request() == request
                && r.unwrap().1.before() == old(self).view(),
            self.history().inputs == old(self).history().inputs.push(Input::Request(request)),
    {
        let ghost before = self.core.view();
        match self.begin_vote(request) {
            None => None,
            Some(operation) => {
                let context = ReplyContext { id: operation.sequence(), image: operation.image(),
                    request, before: Ghost(before) };
                Some((operation, context))
            }
        }
    }

    pub fn finish_rpc(&mut self, context: ReplyContext, written: storage::WrittenVote)
        -> (r: Option<state::VoteReply>)
        requires old(self).invariant(),
        ensures self.invariant(),
            r.is_some() ==> written.id() == context.id() && written.image() == context.image()
                && p::image(self.view().state) == context.image()
                && (r.unwrap().granted ==> r.unwrap().term == context.request().term
                    && self.view().state.voted_for == Some(context.request().candidate)),
            r.is_none() && (written.id() != context.id() || written.image() != context.image())
                ==> self.history() == old(self).history(),
    {
        if !written.matches(context.id, context.image) { return None; }
        if !self.core.pending_matches(context.request) { return None; }
        proof { self.core.view_is_valid(); }
        let ghost before = self.core.view();
        let r = self.release_vote(written);
        proof {
            assert(p::complete_step(before, context.id(), context.image(), self.view(), r));
        }
        r
    }

    pub fn release_vote(&mut self, written: storage::WrittenVote) -> (r: Option<state::VoteReply>)
        requires old(self).invariant(),
        ensures self.invariant(),
            self.history().inputs == old(self).history().inputs.push(Input::Release(written.id(), written.image())),
            self.history().states == old(self).history().states.push(self.view()),
            self.history().outputs == old(self).history().outputs.push(Output::Reply(r)),
            r.is_some() ==> p::image(self.view().state) == written.image(),
            p::complete_step(old(self).view(), written.id(), written.image(), self.view(), r),
    {
        let ghost input = Input::Release(written.id(), written.image());
        let r = written.release_reply(&mut self.core);
        proof { self.core.view_is_valid(); }
        self.journal = Ghost(extend(self.journal@, input, self.core.view(), Output::Reply(r)));
        r
    }
}
}
