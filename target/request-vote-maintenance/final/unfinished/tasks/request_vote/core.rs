//! UNIMPLEMENTED student input. Cargo and Verus import this exact file.
use super::vote_api::*;
use super::vote_api::View;
use vstd::prelude::*;
verus! {
// These fields are a starting declaration, NOT an invariant or solution.
// You may redesign private representation and add auxiliary ghost state.
pub struct Core { state: View, outputs: Ghost<Seq<Reply>>, origin: Ghost<Option<Record>> }
impl Core {
    pub closed spec fn view(&self) -> View { self.state }
    pub closed spec fn replies(&self) -> Seq<Reply> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }

    // Public domain fact, not a supplied inductive Raft invariant. Establish it
    // with your own representation and invariant, including after recovery.
    pub proof fn domain(&self)
        ensures valid_state(self.view().config,self.view().state),
    { assert(false); } // OPEN

    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().replies() == Seq::<Reply>::empty() && out.unwrap().base().is_none(),
    { unreached() } // TODO: implement checked initialization

    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().replies() == Seq::<Reply>::empty() && out.unwrap().base() == Some(record),
    { unreached() } // TODO: implement actual recovery and establish your invariant

    pub fn inspect(&self) -> (out: View) ensures out == self.view()
    { unreached() } // TODO: return the actual state

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.replies() == old(self).replies(), self.base() == old(self).base(),
    { unreached() } // TODO: implement actual receiver and choose useful invariants

    pub fn complete(&mut self, notice: Completion) -> (out: Option<Reply>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.replies() == append_reply(old(self).replies(),out), self.base() == old(self).base(),
    { unreached() } // TODO: authorize only the matching saved reply
}
}
