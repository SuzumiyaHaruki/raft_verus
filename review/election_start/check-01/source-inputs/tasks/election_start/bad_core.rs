//! MAINTENANCE ONLY: deliberately wrong always-grant, constant-term campaign core.
//! It ignores input validity, Busy/Halted and completion identity/success. NOT a solution.
use super::vote_api::*;
use super::vote_api::View;
use vstd::prelude::*;
verus! {
// These fields are a starting declaration, NOT an invariant or solution.
// You may redesign private representation and add auxiliary ghost state.
pub struct Core { state: View, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>> }
impl Core {
    pub closed spec fn view(&self) -> View { self.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }

    // Public domain fact, not a supplied inductive Raft invariant. Establish it
    // with your own representation and invariant, including after recovery.
    pub proof fn domain(tracked &self)
        ensures valid_state(self.view().config,self.view().state),
    { assert(false); } // OPEN

    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none(),
    { Some(Core{state:View{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false},
        outputs:Ghost(Seq::empty()),origin:Ghost(None)}) }

    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record),
    { Some(Core{state:View{config:c,state:record.state,role:Role::Follower,next:0,pending:None,halted:false},
        outputs:Ghost(Seq::empty()),origin:Ghost(Some(record))}) } // DEFECT: resets operation IDs

    pub fn inspect(&self) -> (out: View) ensures out == self.view()
    { self.state }

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    {
        self.state.state = Persistent{term:request.term,voted_for:Some(request.candidate),log:self.state.state.log};
        let w=Write{record:Record{op:0,state:self.state.state},request,
            action:Action::Reply(Reply{node:self.state.config.node,term:request.term,candidate:request.candidate,granted:true})};
        self.state.pending=Some(w);Receive::Save(w)
    } // DEFECT: always grants and reuses op 0

    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(),
    {
        self.state.state=Persistent{term:1,voted_for:Some(self.state.config.node),log:self.state.state.log};
        self.state.role=Role::Candidate;
        let request=Request{term:1,candidate:self.state.config.node,log:self.state.state.log};
        let w=Write{record:Record{op:0,state:self.state.state},request,
            action:Action::Send(Send{config:self.state.config,request})};
        self.state.pending=Some(w);Campaign::Save(w)
    } // DEFECT: constant term, never checks role/Busy/MAX and never increments ID

    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
    {
        let out=match self.state.pending {Some(w)=>Some(w.action),None=>None};
        self.state.pending=None;out
    } // DEFECT: ignores completion ID and success, omits observations
}
}
