//! Shared executable receiver and campaign state, verified with the fixed caller.
use super::vote_api::*;
use super::vote_api::View;
use vstd::prelude::*;
verus! {
pub open spec fn bounded(h: Seq<Choice>, v: View) -> bool {
    forall|i:int| 0 <= i < h.len() ==> (#[trigger] h[i]).node == v.config.node
        && h[i].term <= v.state.term
        && (h[i].term == v.state.term ==> v.state.voted_for == Some(h[i].candidate))
}
pub open spec fn above(h: Seq<Choice>, b: Option<Record>) -> bool {
    b.is_some() ==> forall|i:int| 0 <= i < h.len() ==> (#[trigger] h[i]).term >= b.unwrap().state.term
        && (h[i].term == b.unwrap().state.term ==> b.unwrap().state.voted_for.is_none()
            || b.unwrap().state.voted_for == Some(h[i].candidate))
}
pub open spec fn state_above(v: View, b: Option<Record>) -> bool {
    b.is_some() ==> v.state.term >= b.unwrap().state.term
        && (v.state.term == b.unwrap().state.term && b.unwrap().state.voted_for.is_some()
            ==> v.state.voted_for == b.unwrap().state.voted_for)
}
pub open spec fn pending_ok(v: View) -> bool {
    v.pending.is_some() ==> {
        let w = v.pending.unwrap();
        w.record.state == v.state && w.record.op < v.next && !v.halted
        && match w.action {
            Action::Reply(r) => r.node == v.config.node && r.term == v.state.term
                && (r.granted ==> v.state.voted_for == Some(r.candidate)),
            Action::Send(s) => s.config == v.config && s.request.term == v.state.term
                && s.request.candidate == v.config.node && s.request.log == v.state.log
                && v.state.voted_for == Some(v.config.node),
        }
    }
}
pub struct Core { state: View, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>> }
impl Core {
    pub closed spec fn view(&self) -> View { self.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        valid_state(self.state.config,self.state.state) && pending_ok(self.state)
        && unique(self.outputs@) && bounded(self.outputs@, self.state)
        && above(self.outputs@,self.origin@) && state_above(self.state,self.origin@)
    }
    pub proof fn facts(tracked &self)
        ensures valid_state(self.view().config,self.view().state), pending_ok(self.view()),
            unique(self.choices()), bounded(self.choices(),self.view()),
            above(self.choices(),self.base()), state_above(self.view(),self.base()),
    { use_type_invariant(self); }
    pub proof fn domain(tracked &self)
        ensures valid_state(self.view().config,self.view().state),
    { self.facts(); }

    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none(),
    {
        if !check_state(c,initial.state) || (initial.state.term == 0 && !matches!(initial.role,Role::Follower)) { return None; }
        Some(Core { state: View{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false}, outputs:Ghost(Seq::empty()),origin:Ghost(None) })
    }
    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record),
    {
        if !check_state(c,record.state) || record.op == u64::MAX { return None; }
        Some(Core { state:View{config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false},outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)) })
    }
    pub fn inspect(&self) -> (out: View) ensures out == self.view() { self.state }

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        let v = self.state;
        if v.halted { return Receive::Reject(Fault::Halted); }
        if v.pending.is_some() { return Receive::Reject(Fault::Busy); }
        if request.term == 0 || request.candidate >= v.config.members
            || ((request.log.index == 0) != (request.log.term == 0)) || request.log.term > request.term {
            return Receive::Reject(Fault::Invalid);
        }
        if v.next == u64::MAX { return Receive::Reject(Fault::Exhausted); }
        let eligible = request.term >= v.state.term
            && (request.log.term > v.state.log.term || (request.log.term == v.state.log.term && request.log.index >= v.state.log.index))
            && (request.term > v.state.term || v.state.voted_for.is_none() || v.state.voted_for == Some(request.candidate));
        let state = Persistent {term:if request.term > v.state.term {request.term} else {v.state.term},
            voted_for:if eligible {Some(request.candidate)} else if request.term > v.state.term {None} else {v.state.voted_for},log:v.state.log};
        let w = Write{record:Record{op:v.next,state},request,action:Action::Reply(Reply{node:v.config.node,term:state.term,candidate:request.candidate,granted:eligible})};
        self.state = View{config:v.config,state,role:if request.term > v.state.term {Role::Follower} else {v.role},next:v.next+1,pending:Some(w),halted:false};
        Receive::Save(w)
    }
    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        let v = self.state;
        if v.halted { return Campaign::Reject(Fault::Halted); }
        if v.pending.is_some() { return Campaign::Reject(Fault::Busy); }
        if matches!(v.role,Role::Leader) { return Campaign::Reject(Fault::Leader); }
        if v.state.term == u64::MAX { return Campaign::Reject(Fault::TermExhausted); }
        if v.next == u64::MAX { return Campaign::Reject(Fault::Exhausted); }
        let state = Persistent{term:v.state.term+1,voted_for:Some(v.config.node),log:v.state.log};
        let request = Request{term:state.term,candidate:v.config.node,log:state.log};
        let w = Write{record:Record{op:v.next,state},request,action:Action::Send(Send{config:v.config,request})};
        self.state = View{config:v.config,state,role:Role::Candidate,next:v.next+1,pending:Some(w),halted:false};
        proof { append_choice(self.outputs@, self.state, self.origin@, choice(self.state)); }
        self.outputs = Ghost(self.outputs@.push(choice(self.state)));
        Campaign::Save(w)
    }
    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice), self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        if let Some(w) = self.state.pending {
            if w.record.op == notice.op {
                self.state.pending = None;
                if !notice.success { self.state.halted = true; return None; }
                if let Action::Reply(r) = w.action {
                    if r.granted {
                        proof { append_choice(self.outputs@,self.state,self.origin@,Choice{node:r.node,term:r.term,candidate:r.candidate}); }
                        self.outputs = Ghost(self.outputs@.push(Choice{node:r.node,term:r.term,candidate:r.candidate}));
                    }
                }
                return Some(w.action);
            }
        }
        None
    }
}
fn check_state(c: Config, s: Persistent) -> (out: bool)
    ensures out == valid_state(c,s),
{
    c.members > 0 && c.node < c.members && ((s.log.index == 0) == (s.log.term == 0))
        && s.log.term <= s.term && match s.voted_for {None => true, Some(n) => s.term > 0 && n < c.members}
}
proof fn append_choice(h: Seq<Choice>, v: View, b: Option<Record>, x: Choice)
    requires unique(h), bounded(h,v), above(h,b), state_above(v,b),
        x.node == v.config.node, x.term == v.state.term, v.state.voted_for == Some(x.candidate),
    ensures unique(h.push(x)), bounded(h.push(x),v), above(h.push(x),b),
{
    assert forall|i:int,j:int| 0 <= i < h.push(x).len() && 0 <= j < h.push(x).len()
        && (#[trigger] h.push(x)[i]).node == (#[trigger] h.push(x)[j]).node && h.push(x)[i].term == h.push(x)[j].term
        implies h.push(x)[i].candidate == h.push(x)[j].candidate by {
        if i < h.len() && j < h.len() { assert(h[i].candidate == h[j].candidate); }
    }
}
}
