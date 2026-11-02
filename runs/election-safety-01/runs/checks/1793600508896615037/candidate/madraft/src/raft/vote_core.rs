//! Shared executable election authority, imported by Cargo and Verus.
use super::vote_api::*;
use super::vote_api::View as StateView;
use vstd::prelude::*;
verus! {
// These fields are a starting declaration, NOT an invariant or solution.
// You may redesign private representation and add auxiliary ghost state.
pub struct Core { state: StateView, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>>, voters: Vec<u64> }
impl Core {
    #[verifier::type_invariant]
    spec fn inv(self) -> bool { self.voters@.no_duplicates() && self.voters.len() <= u64::MAX && (forall|n:u64| self.voters@.contains(n) ==> n<self.state.config.members) }
    fn valid_persistent(c:Config,s:Persistent) -> (ok:bool) ensures ok==valid_state(c,s) {
        c.members>0 && c.node<c.members && ((s.log.index==0)==(s.log.term==0)) && s.log.term<=s.term && match s.voted_for {Some(n)=>s.term>0 && n<c.members,None=>true}
    }
    fn valid_req(c:Config,r:Request) -> (ok:bool) ensures ok==valid_request(c,r) {
        r.term>0 && r.candidate<c.members && ((r.log.index==0)==(r.log.term==0)) && r.log.term<=r.term
    }

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
    {
        if !Self::valid_persistent(c,initial.state) || (initial.state.term == 0 && !matches!(initial.role,Role::Follower)) { return None; }
        Some(Core{state:StateView{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(None),voters:Vec::new()})
    }

    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !Self::valid_persistent(c,record.state) || record.op == u64::MAX { return None; }
        Some(Core{state:StateView{config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)),voters:Vec::new()})
    }

    pub fn inspect(&self) -> (out: StateView) ensures out == self.view()
    { self.state }

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
    { proof {use_type_invariant(&*self);}
        let v=self.state;
        if v.halted { return Receive::Reject(Fault::Halted); }
        if v.pending.is_some() { return Receive::Reject(Fault::Busy); }
        if !Self::valid_req(v.config,request) { return Receive::Reject(Fault::Invalid); }
        if v.next == u64::MAX {
            if request.term > v.state.term { self.state.state.term=request.term; self.state.state.voted_for=None; self.state.role=Role::Follower; self.state.round=None; self.state.self_saved=false; self.state.halted=true; self.voters=Vec::new(); }
            return Receive::Reject(Fault::Exhausted);
        }
        let fresh=request.log.term>v.state.log.term || (request.log.term==v.state.log.term && request.log.index>=v.state.log.index);
        let granted=request.term>=v.state.term && fresh && (request.term>v.state.term || v.state.voted_for.is_none() || v.state.voted_for==Some(request.candidate));
        let state=Persistent{term:if request.term>v.state.term {request.term} else {v.state.term}, voted_for:if granted {Some(request.candidate)} else if request.term>v.state.term {None} else {v.state.voted_for},log:v.state.log};
        let w=Write{record:Record{op:v.next,state},request:Some(request),action:Action::Reply(Reply{node:v.config.node,term:state.term,candidate:request.candidate,granted})};
        self.state.state=state; self.state.next=v.next+1; self.state.pending=Some(w);
        if request.term>v.state.term {self.state.role=Role::Follower;self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();}
        Receive::Save(w)
    }

    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
    { proof {use_type_invariant(&*self);}
        let v=self.state;
        if v.halted {return Campaign::Reject(Fault::Halted);}
        if v.pending.is_some() {return Campaign::Reject(Fault::Busy);}
        if matches!(v.role,Role::Leader) {return Campaign::Reject(Fault::Leader);}
        if v.state.term==u64::MAX {return Campaign::Reject(Fault::TermExhausted);}
        if v.next==u64::MAX {return Campaign::Reject(Fault::Exhausted);}
        let state=Persistent{term:v.state.term+1,voted_for:Some(v.config.node),log:v.state.log};
        let request=Request{term:state.term,candidate:v.config.node,log:state.log};
        let w=Write{record:Record{op:v.next,state},request:Some(request),action:Action::Send(Send{config:v.config,request})};
        self.state=StateView{config:v.config,state,role:Role::Candidate,next:v.next+1,pending:Some(w),halted:false,round:Some(state.term),self_saved:false};
        self.voters=Vec::new();
        proof {self.outputs@=self.outputs@.push(choice(self.view()));}
        Campaign::Save(w)
    }

    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
    { proof {use_type_invariant(&*self);}
        let v=self.state;
        let Some(w)=v.pending else {return None;};
        if w.record.op!=notice.op {return None;}
        self.state.pending=None;
        if !notice.success {self.state.halted=true;self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();return None;}
        match w.action {
            Action::Send(_) => {self.state.self_saved=true;let mut voters=Vec::new();voters.push(v.config.node);self.voters=voters;},
            Action::Reply(r) => {proof {if r.granted {self.outputs@=self.outputs@.push(Choice{node:r.node,term:r.term,candidate:r.candidate});}}},
            _ => {},
        }
        Some(w.action)
    }
    pub fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        ensures out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    { proof {use_type_invariant(&*self);}
        let v=self.state;
        if v.halted {return Collect::Reject(Fault::Halted);}
        if v.pending.is_some() {return Collect::Reject(Fault::Busy);}
        let s=context.sent;
        if s.config.node!=v.config.node || s.config.members!=v.config.members || context.peer>=v.config.members || context.peer==v.config.node || s.request.candidate!=v.config.node || !Self::valid_req(v.config,s.request) {return Collect::Reject(Fault::Invalid);}
        let RpcResult::Reply{term,granted}=result else {return Collect::Ignored;};
        if term>v.state.term {
            self.state.state=Persistent{term,voted_for:None,log:v.state.log};self.state.role=Role::Follower;self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();
            if v.next==u64::MAX {self.state.halted=true;return Collect::Reject(Fault::Exhausted);}
            let w=Write{record:Record{op:v.next,state:self.state.state},request:None,action:Action::Persisted};
            self.state.next=v.next+1;self.state.pending=Some(w);return Collect::Save(w);
        }
        if matches!(v.role,Role::Candidate) && v.self_saved && v.round==Some(v.state.term) && granted && term==v.state.term && s.request.term==term && s.request.log.term==v.state.log.term && s.request.log.index==v.state.log.index && !self.has_vote(context.peer) {
            let mut voters=self.voters.clone();voters.push(context.peer);self.voters=voters;return Collect::Counted;
        }
        Collect::Ignored
    }

    // No external count, boolean or certificate: inspect this Core's own votes.
    pub fn try_become_leader(&mut self) -> (elected: bool)
        ensures elected == promotion_allowed(old(self).view(),old(self).votes()),
            self.view() == promotion_after(old(self).view(),old(self).votes()),
            self.votes() == old(self).votes(), self.choices() == old(self).choices(),
            self.base() == old(self).base(),
    { proof {use_type_invariant(&*self);}
        let v=self.state;
        if !v.halted && v.pending.is_none() && matches!(v.role,Role::Candidate) && v.self_saved && v.round==Some(v.state.term) && v.state.voted_for==Some(v.config.node) && self.has_vote(v.config.node) && v.config.members>0 && v.config.node<v.config.members && self.vote_count()>v.config.members/2 {
            self.state.role=Role::Leader;true
        } else {false}
    }

    pub fn vote_count(&self) -> (out: u64) ensures out as nat == self.votes().len()
    { proof { use_type_invariant(self); self.voters@.unique_seq_to_set(); } self.voters.len() as u64 }

    pub fn has_vote(&self, member: u64) -> (out: bool) ensures out == self.votes().contains(member)
    {
        let mut i:usize=0;
        while i<self.voters.len()
            invariant i<=self.voters.len(), forall|j:int| 0<=j<i ==> self.voters@[j]!=member,
            decreases self.voters.len()-i,
        {
            if self.voters[i]==member {return true;} i+=1;
        }
        false
    }

}
}
