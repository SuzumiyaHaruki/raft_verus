//! Synchronous RequestVote decision and persistence authorization core.
use super::vote_api::*;
use super::vote_api::View;
use vstd::prelude::*;
verus! {
pub struct Core { state: View, outputs: Ghost<Seq<Reply>>, origin: Ghost<Option<Record>> }

pub open spec fn bounded(a: Reply, s: Persistent) -> bool {
    a.granted ==> a.term <= s.term && (a.term == s.term ==> s.voted_for == Some(a.candidate))
}
pub open spec fn rooted(a: Reply, b: Option<Record>) -> bool {
    b.is_some() && a.granted ==> a.term >= b.unwrap().state.term
        && (a.term == b.unwrap().state.term && b.unwrap().state.voted_for.is_some()
            ==> b.unwrap().state.voted_for == Some(a.candidate))
}
pub open spec fn core_ok(v: View, h: Seq<Reply>, b: Option<Record>) -> bool {
    &&& valid_state(v.config,v.state)
    &&& unique(h)
    &&& forall|i:int| 0 <= i < h.len() ==> bounded(#[trigger] h[i],v.state) && rooted(h[i],b)
    &&& (b.is_some() ==> v.state.term >= b.unwrap().state.term
        && (v.state.term == b.unwrap().state.term && b.unwrap().state.voted_for.is_some()
            ==> v.state.voted_for == b.unwrap().state.voted_for))
    &&& (v.pending.is_some() ==> {
        let w = v.pending.unwrap();
        w.record.state == v.state && w.record.op < u64::MAX && w.record.op + 1 == v.next
        && w.reply.term == v.state.term && bounded(w.reply,v.state) && rooted(w.reply,b)
    })
}
impl Core {
    pub closed spec fn view(&self) -> View { self.state }
    pub closed spec fn replies(&self) -> Seq<Reply> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool { core_ok(self.state,self.outputs@,self.origin@) }
    pub proof fn facts(tracked &self) ensures core_ok(self.view(),self.replies(),self.base())
    { use_type_invariant(self); }
    pub proof fn domain(tracked &self) ensures valid_state(self.view().config,self.view().state)
    { self.facts(); }

    fn valid(c: Config, s: Persistent) -> (ok: bool)
        ensures ok == valid_state(c,s),
    {
        c.members > 0 && c.node < c.members && ((s.log.index == 0) == (s.log.term == 0))
        && s.log.term <= s.term && match s.voted_for { Some(x) => s.term > 0 && x < c.members, None => true }
    }
    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().replies() == Seq::<Reply>::empty() && out.unwrap().base().is_none(),
    {
        if !Self::valid(c,initial.state) || (initial.state.term == 0 && !matches!(initial.role,Role::Follower)) { return None; }
        Some(Core { state: View { config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false },
            outputs:Ghost(Seq::empty()),origin:Ghost(None) })
    }
    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().replies() == Seq::<Reply>::empty() && out.unwrap().base() == Some(record),
    {
        if !Self::valid(c,record.state) || record.op == u64::MAX { return None; }
        Some(Core { state: View { config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false },
            outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)) })
    }
    pub fn inspect(&self) -> (out: View) ensures out == self.view() { self.state }
    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.replies() == old(self).replies(), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        if self.state.halted { return Receive::Reject(Fault::Halted); }
        if self.state.pending.is_some() { return Receive::Reject(Fault::Busy); }
        if request.term == 0 || request.candidate >= self.state.config.members
            || ((request.log.index == 0) != (request.log.term == 0)) || request.log.term > request.term {
            return Receive::Reject(Fault::Invalid);
        }
        if self.state.next == u64::MAX { return Receive::Reject(Fault::Exhausted); }
        let ghost previous = self.state;
        let higher = request.term > self.state.state.term;
        let available = match self.state.state.voted_for { None => true, Some(x) => x == request.candidate };
        let grant = request.term >= self.state.state.term && (higher || available)
            && (request.log.term > self.state.state.log.term
                || (request.log.term == self.state.state.log.term && request.log.index >= self.state.state.log.index));
        let s = Persistent { term: if higher { request.term } else { self.state.state.term },
            voted_for: if grant { Some(request.candidate) } else if higher { None } else { self.state.state.voted_for },
            log:self.state.state.log };
        let w = Write { record:Record{op:self.state.next,state:s},request,
            reply:Reply{node:self.state.config.node,term:s.term,candidate:request.candidate,granted:grant} };
        self.state = View { config:self.state.config,state:s,role:if higher {Role::Follower} else {self.state.role},
            next:self.state.next+1,pending:Some(w),halted:false };
        proof {
            assert forall|i:int| 0 <= i < self.outputs@.len() implies bounded(#[trigger] self.outputs@[i],s) && rooted(self.outputs@[i],self.origin@) by {
                assert(bounded(self.outputs@[i],previous.state));
            }
        }
        Receive::Save(w)
    }
    pub fn complete(&mut self, notice: Completion) -> (out: Option<Reply>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.replies() == append_reply(old(self).replies(),out), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        if let Some(w) = self.state.pending {
            if w.record.op == notice.op {
                self.state.pending = None;
                if notice.success {
                    proof {
                        let h = self.outputs@;
                        assert forall|i:int,j:int| 0 <= i < h.push(w.reply).len() && 0 <= j < h.push(w.reply).len()
                            && (#[trigger] h.push(w.reply)[i]).granted && (#[trigger] h.push(w.reply)[j]).granted
                            && h.push(w.reply)[i].node == h.push(w.reply)[j].node
                            && h.push(w.reply)[i].term == h.push(w.reply)[j].term
                            implies h.push(w.reply)[i].candidate == h.push(w.reply)[j].candidate by {
                            if i < h.len() && j < h.len() { assert(h[i].candidate == h[j].candidate); }
                            else if i < h.len() { assert(bounded(h[i],self.state.state)); }
                            else if j < h.len() { assert(bounded(h[j],self.state.state)); }
                        }
                        self.outputs@ = h.push(w.reply);
                    }
                    return Some(w.reply);
                } else { self.state.halted = true; }
            }
        }
        None
    }
}
}
