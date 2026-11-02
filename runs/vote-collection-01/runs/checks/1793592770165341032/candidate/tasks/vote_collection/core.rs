//! UNIMPLEMENTED student input. Cargo and Verus import this exact file.
use super::vote_api::*;
use super::vote_api::View as StateView;
use vstd::prelude::*;
verus! {
// These fields are a starting declaration, NOT an invariant or solution.
// You may redesign private representation and add auxiliary ghost state.
pub struct Core { state: StateView, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>>, voters: Vec<u64> }
impl Core {
    pub closed spec fn view(&self) -> StateView { self.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }

    pub closed spec fn votes(&self) -> Set<u64> { self.voters@.to_set() }

    // Public domain fact, not a supplied inductive Raft invariant. Establish it
    // with your own representation and invariant, including after recovery.
    pub proof fn domain(tracked &self)
        ensures valid_state(self.view().config,self.view().state), voters_domain(self.view(),self.votes()),
    { assert(false); } // OPEN

    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none() && out.unwrap().votes() == Set::<u64>::empty(),
    { unreached() } // TODO: implement checked initialization

    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    { unreached() } // TODO: implement actual recovery and establish your invariant

    pub fn inspect(&self) -> (out: StateView) ensures out == self.view()
    { unreached() } // TODO: return the actual state

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
    { unreached() } // TODO: implement actual receiver and choose useful invariants

    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
    { unreached() } // TODO: actual checked increment/self-choice/save, no send authorization here

    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
    { unreached() } // TODO: authorize only the matching saved reply OR campaign send
    pub fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        ensures out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    { unreached() } // OPEN: actual reply handling, including higher old replies.

    pub fn vote_count(&self) -> (out: u64) ensures out as nat == self.votes().len()
    { unreached() } // OPEN: executable query, not a ghost-only tally.

    pub fn has_vote(&self, member: u64) -> (out: bool) ensures out == self.votes().contains(member)
    { unreached() }

}
}
