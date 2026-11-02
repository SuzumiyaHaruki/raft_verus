//! Executable receiver, campaign, and reply collection share this state.
use super::vote_api::*;
use super::vote_api::View as StateView;
use vstd::prelude::*;
verus! {
pub struct Core { state: StateView, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>>, voters: Vec<u64> }
impl Core {
    pub closed spec fn view(&self) -> StateView { self.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }
    pub closed spec fn votes(&self) -> Set<u64> { self.voters@.to_set() }

    #[verifier::type_invariant]
    closed spec fn wf(&self) -> bool {
        &&& valid_state(self.state.config,self.state.state)
        &&& self.voters@.no_duplicates()
        &&& voters_domain(self.state,self.votes())
        &&& unique(self.choices())
        &&& forall|i:int| 0<=i<self.choices().len() ==> {
            let c=#[trigger] self.choices()[i];
            c.node==self.state.config.node && c.term<=self.state.state.term
            && (c.term==self.state.state.term ==> self.state.state.voted_for==Some(c.candidate))
        }
        &&& self.base().is_some() ==> {
            let b=self.base().unwrap();
            valid_record(self.state.config,b) && b.state.term<=self.state.state.term
            && (b.state.term==self.state.state.term && b.state.voted_for.is_some() ==> self.state.state.voted_for==b.state.voted_for)
            && forall|i:int| 0<=i<self.choices().len() ==> {
                let c=#[trigger] self.choices()[i]; c.term>=b.state.term
                && (c.term==b.state.term && b.state.voted_for.is_some() ==> Some(c.candidate)==b.state.voted_for)
            }
        }
        &&& self.state.pending.is_some() ==> {
            let w=self.state.pending.unwrap();
            w.record.state==self.state.state && w.record.op<self.state.next
            && match w.action {
                Action::Reply(r) => r.node==self.state.config.node && r.term==self.state.state.term
                    && (r.granted ==> self.state.state.voted_for==Some(r.candidate)),
                Action::Send(s) => s.config==self.state.config && s.request==Request{term:self.state.state.term,candidate:self.state.config.node,log:self.state.state.log}
                    && self.state.state.voted_for==Some(self.state.config.node)
                    && self.state.role==Role::Candidate && self.state.round==Some(self.state.state.term),
                Action::Persisted => true,
            }
        }
    }
    pub proof fn domain(tracked &self)
        ensures valid_state(self.view().config,self.view().state), voters_domain(self.view(),self.votes()),
    { use_type_invariant(self); }
    pub proof fn history(tracked &self)
        ensures unique(self.choices()),
            forall|i:int| 0<=i<self.choices().len() ==> {
                let c=#[trigger] self.choices()[i]; c.term<=self.view().state.term
                && (c.term==self.view().state.term ==> self.view().state.voted_for==Some(c.candidate)) },
            self.base().is_some() ==> forall|i:int| 0<=i<self.choices().len() ==> {
                let c=#[trigger] self.choices()[i]; c.term>=self.base().unwrap().state.term
                && (c.term==self.base().unwrap().state.term && self.base().unwrap().state.voted_for.is_some() ==> Some(c.candidate)==self.base().unwrap().state.voted_for) },
    { use_type_invariant(self); }

    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none() && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !valid(c,initial.state) || (initial.state.term==0 && !matches!(initial.role,Role::Follower)) { return None; }
        Some(Core{state:StateView{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(None),voters:Vec::new()})
    }
    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !valid(c,record.state) || record.op==u64::MAX { return None; }
        Some(Core{state:StateView{config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)),voters:Vec::new()})
    }
    pub fn inspect(&self) -> (out: StateView) ensures out == self.view() { self.state }

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
    {
        proof { use_type_invariant(&*self); }
        if self.state.halted { return Receive::Reject(Fault::Halted); }
        if self.state.pending.is_some() { return Receive::Reject(Fault::Busy); }
        if !request_valid(self.state.config,request) { return Receive::Reject(Fault::Invalid); }
        let higher=request.term>self.state.state.term;
        if self.state.next==u64::MAX {
            if higher { self.invalidate(request.term); self.state.halted=true; }
            return Receive::Reject(Fault::Exhausted);
        }
        let grant=request.term>=self.state.state.term
            && (request.log.term>self.state.state.log.term || (request.log.term==self.state.state.log.term && request.log.index>=self.state.state.log.index))
            && (higher || self.state.state.voted_for.is_none() || self.state.state.voted_for==Some(request.candidate));
        if higher { self.invalidate(request.term); }
        if grant { self.state.state.voted_for=Some(request.candidate); }
        let w=Write{record:Record{op:self.state.next,state:self.state.state},request:Some(request),action:Action::Reply(Reply{node:self.state.config.node,term:self.state.state.term,candidate:request.candidate,granted:grant})};
        self.state.next=self.state.next+1; self.state.pending=Some(w);
        Receive::Save(w)
    }
    fn invalidate(&mut self, term:u64)
        requires term>old(self).view().state.term, old(self).view().pending.is_none(),
        ensures self.view()==(StateView{state:Persistent{term,voted_for:None,log:old(self).view().state.log},role:Role::Follower,round:None,self_saved:false,..old(self).view()}),
            self.votes()==Set::<u64>::empty(),self.choices()==old(self).choices(),self.base()==old(self).base(),
    {
        self.state.state.term=term;self.state.state.voted_for=None;self.state.role=Role::Follower;
        self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();
    }
    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
    {
        proof { use_type_invariant(&*self); }
        if self.state.halted { return Campaign::Reject(Fault::Halted); }
        if self.state.pending.is_some() { return Campaign::Reject(Fault::Busy); }
        if matches!(self.state.role,Role::Leader) { return Campaign::Reject(Fault::Leader); }
        if self.state.state.term==u64::MAX { return Campaign::Reject(Fault::TermExhausted); }
        if self.state.next==u64::MAX { return Campaign::Reject(Fault::Exhausted); }
        self.state.state.term=self.state.state.term+1;
        self.state.state.voted_for=Some(self.state.config.node);self.state.role=Role::Candidate;
        self.state.round=Some(self.state.state.term);self.state.self_saved=false;self.voters=Vec::new();
        let request=Request{term:self.state.state.term,candidate:self.state.config.node,log:self.state.state.log};
        let w=Write{record:Record{op:self.state.next,state:self.state.state},request:Some(request),action:Action::Send(Send{config:self.state.config,request})};
        self.state.next=self.state.next+1;self.state.pending=Some(w);
        proof { self.outputs@=self.outputs@.push(choice(self.state)); }
        Campaign::Save(w)
    }
    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
    {
        proof { use_type_invariant(&*self); }
        if let Some(w)=self.state.pending {
            if w.record.op==notice.op {
                self.state.pending=None;
                if !notice.success { self.state.halted=true;self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();return None; }
                if let Action::Send(_)=w.action {
                    self.state.self_saved=true;let mut voters=Vec::new();voters.push(self.state.config.node);self.voters=voters;
                }
                proof { self.outputs@=append_action(self.outputs@,Some(w.action)); }
                return Some(w.action);
            }
        }
        None
    }
    pub fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        ensures out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        if self.state.halted { return Collect::Reject(Fault::Halted); }
        if self.state.pending.is_some() { return Collect::Reject(Fault::Busy); }
        if context.sent.config.node!=self.state.config.node || context.sent.config.members!=self.state.config.members
            || context.peer>=self.state.config.members || context.peer==self.state.config.node
            || context.sent.request.candidate!=self.state.config.node || !request_valid(self.state.config,context.sent.request) { return Collect::Reject(Fault::Invalid); }
        match result {
            RpcResult::Error => Collect::Ignored,
            RpcResult::Reply{term,granted} => {
                if term>self.state.state.term {
                    self.invalidate(term);
                    if self.state.next==u64::MAX { self.state.halted=true;return Collect::Reject(Fault::Exhausted); }
                    let w=Write{record:Record{op:self.state.next,state:self.state.state},request:None,action:Action::Persisted};
                    self.state.next=self.state.next+1;self.state.pending=Some(w);Collect::Save(w)
                } else if matches!(self.state.role,Role::Candidate) && self.state.self_saved && self.state.round==Some(self.state.state.term)
                    && granted && term==self.state.state.term && context.sent.request.term==term
                    && context.sent.request.log.term==self.state.state.log.term && context.sent.request.log.index==self.state.state.log.index && !self.has_vote(context.peer) {
                    let mut voters=Vec::new();core::mem::swap(&mut self.voters,&mut voters);voters.push(context.peer);self.voters=voters;Collect::Counted
                } else { Collect::Ignored }
            }
        }
    }
    pub fn vote_count(&self) -> (out: u64) ensures out as nat == self.votes().len()
    { proof { use_type_invariant(self); self.voters@.unique_seq_to_set(); } self.voters.len() as u64 }
    pub fn has_vote(&self, member: u64) -> (out: bool) ensures out == self.votes().contains(member)
    {
        let mut i:usize=0;
        while i<self.voters.len()
            invariant i<=self.voters.len(), forall|j:int| 0<=j<i ==> self.voters@[j]!=member,
            decreases self.voters.len()-i,
        { if self.voters[i]==member { return true; } i+=1; }
        false
    }
}
fn valid(c:Config,s:Persistent) -> (out:bool) ensures out==valid_state(c,s) {
    c.members>0 && c.node<c.members && (s.log.index==0)==(s.log.term==0) && s.log.term<=s.term
        && match s.voted_for {Some(n)=>s.term>0 && n<c.members,None=>true}
}
fn request_valid(c:Config,r:Request) -> (out:bool) ensures out==valid_request(c,r) {
    r.term>0 && r.candidate<c.members && (r.log.index==0)==(r.log.term==0) && r.log.term<=r.term
}
}
