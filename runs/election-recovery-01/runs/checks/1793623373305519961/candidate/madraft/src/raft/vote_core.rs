//! Executable election and completed-write recovery core.
use super::vote_api::*;
use super::vote_api::View as StateView;
use vstd::prelude::*;
use vstd::seq_lib::*;
use vstd::set_lib::*;
verus! {
broadcast use vstd::seq_lib::group_seq_properties;
broadcast use vstd::set::group_set_axioms;
// These fields are a starting declaration, NOT an invariant or solution.
// You may redesign private representation and add auxiliary ghost state.
pub struct Core { data: Data }
impl Core {
    #[verifier::type_invariant]
    closed spec fn wf(&self) -> bool { self.data.wf() }
    pub closed spec fn view(&self) -> StateView { self.data.view() }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.data.choices() }
    pub closed spec fn base(&self) -> Option<Record> { self.data.base() }
    pub closed spec fn votes(&self) -> Set<u64> { self.data.votes() }
    pub proof fn domain(tracked &self)
        ensures valid_state(self.view().config,self.view().state), voters_domain(self.view(),self.votes()),
    { use_type_invariant(self); self.data.domain() }
    pub fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none() && out.unwrap().votes() == Set::<u64>::empty(),
    { let d=Data::new(c,initial); match d {Some(data)=>Some(Core{data}),None=>None} }
    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    { let d=Data::recover(c,record); match d {Some(data)=>Some(Core{data}),None=>None} }
    pub fn inspect(&self) -> (out: StateView) ensures out == self.view()
    { proof { use_type_invariant(&*self); } self.data.inspect() }
    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
    { proof { use_type_invariant(&*self); } self.data.receive(request) }
    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
    { proof { use_type_invariant(&*self); } self.data.start_election() }
    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
    { proof { use_type_invariant(&*self); } self.data.complete(notice) }
    pub fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        ensures out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    { proof { use_type_invariant(&*self); } self.data.collect(context,result) }
    pub fn try_become_leader(&mut self) -> (elected: bool)
        ensures elected == promotion_allowed(old(self).view(),old(self).votes()),
            self.view() == promotion_after(old(self).view(),old(self).votes()),
            self.votes() == old(self).votes(), self.choices() == old(self).choices(),
            self.base() == old(self).base(),
    { proof { use_type_invariant(&*self); } self.data.try_become_leader() }
    pub fn vote_count(&self) -> (out: u64) ensures out as nat == self.votes().len()
    { proof { use_type_invariant(&*self); } self.data.vote_count() }
    pub fn has_vote(&self, member: u64) -> (out: bool) ensures out == self.votes().contains(member)
    { proof { use_type_invariant(&*self); } self.data.has_vote(member) }
}
struct Data { state: StateView, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>>, voters: Vec<u64> }
impl Data {
    closed spec fn wf(&self) -> bool {
        valid_state(self.state.config,self.state.state)
        && voters_domain(self.state,self.votes())
        && self.voters@.no_duplicates()
        && self.voters.len() <= u64::MAX
        && (self.state.pending.is_some() ==> {
            let w=self.state.pending.unwrap();
            w.record.state==self.state.state && w.record.op<self.state.next
            && (w.action is Send ==> self.state.role==Role::Candidate && self.state.round==Some(self.state.state.term) && !self.state.halted)
        })
    }
    closed spec fn view(&self) -> StateView { self.state }
    closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    closed spec fn base(&self) -> Option<Record> { self.origin@ }

    closed spec fn votes(&self) -> Set<u64> { self.voters@.to_set() }

    // Public domain fact, not a supplied inductive Raft invariant. Establish it
    // with your own representation and invariant, including after recovery.
    proof fn domain(tracked &self)
        requires self.wf(), ensures valid_state(self.view().config,self.view().state), voters_domain(self.view(),self.votes()),
    { assert(self.wf()); }

    fn new(c: Config, initial: Initial) -> (out: Option<Self>)
        ensures out.is_some() ==> out.unwrap().wf(),
            out.is_some() <==> valid_initial(c,initial),
            out.is_some() ==> out.unwrap().view() == fresh_view(c,initial)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base().is_none() && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !state_valid(c,initial.state) || (initial.state.term==0 && !matches!(initial.role,Role::Follower)) { return None; }
        Some(Data{state:StateView{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(None),voters:Vec::new()})
    }

    fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() ==> out.unwrap().wf(),
            out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !state_valid(c,record.state) || record.op==u64::MAX { return None; }
        Some(Data{state:StateView{config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)),voters:Vec::new()})
    }

    fn inspect(&self) -> (out: StateView) requires self.wf(), ensures out == self.view()
    { self.state }

    fn receive(&mut self, request: Request) -> (out: Receive)
        requires old(self).wf(),
        ensures self.wf(),
            out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
        no_unwind
    {
        proof { assert(self.wf()); }
        let v=self.state;
        if v.halted {return Receive::Reject(Fault::Halted);}
        if v.pending.is_some() {return Receive::Reject(Fault::Busy);}
        if !request_valid(v.config,request) {return Receive::Reject(Fault::Invalid);}
        if v.next==u64::MAX {
            if request.term>v.state.term {self.demote(request.term);self.state.halted=true;}
            return Receive::Reject(Fault::Exhausted);
        }
        let higher=request.term>v.state.term;
        let grant=request.term>=v.state.term && (request.log.term>v.state.log.term || (request.log.term==v.state.log.term && request.log.index>=v.state.log.index)) && (higher || v.state.voted_for.is_none() || v.state.voted_for==Some(request.candidate));
        let state=Persistent{term:if higher {request.term} else {v.state.term},voted_for:if grant {Some(request.candidate)} else if higher {None} else {v.state.voted_for},log:v.state.log};
        let w=Write{record:Record{op:v.next,state},request:Some(request),action:Action::Reply(Reply{node:v.config.node,term:state.term,candidate:request.candidate,granted:grant})};
        if higher {self.demote(request.term);}
        self.state.state=state;self.state.next=v.next+1;self.state.pending=Some(w);
        Receive::Save(w)
    }

    fn start_election(&mut self) -> (out: Campaign)
        requires old(self).wf(),
        ensures self.wf(),
            out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
        no_unwind
    {
        proof { assert(self.wf()); }
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
        proof { self.outputs@=self.outputs@.push(choice(self.view())); }
        Campaign::Save(w)
    }

    fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        requires old(self).wf(),
        ensures self.wf(),
            out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
        no_unwind
    {
        proof { assert(self.wf()); }
        let v=self.state;
        if let Some(w)=v.pending {
            if w.record.op==notice.op {
                self.state.pending=None;
                if !notice.success {
                    self.state.halted=true;self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();return None;
                }
                if matches!(w.action,Action::Send(_)) {self.state.self_saved=true;let mut own=Vec::new();own.push(v.config.node);self.voters=own;}
                proof { self.outputs@=append_action(self.outputs@,Some(w.action)); }
                return Some(w.action);
            }
        }
        None
    }
    fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        requires old(self).wf(),
        ensures self.wf(),
            out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
        no_unwind
    {
        proof { assert(self.wf()); }
        let v=self.state;
        if v.halted {return Collect::Reject(Fault::Halted);}
        if v.pending.is_some() {return Collect::Reject(Fault::Busy);}
        if !context_valid(v,context) {return Collect::Reject(Fault::Invalid);}
        if let RpcResult::Reply{term,granted}=result {
            if term>v.state.term {
                self.demote(term);
                if v.next==u64::MAX {self.state.halted=true;return Collect::Reject(Fault::Exhausted);}
                let w=Write{record:Record{op:v.next,state:self.state.state},request:None,action:Action::Persisted};
                self.state.next=v.next+1;self.state.pending=Some(w);return Collect::Save(w);
            }
            if matches!(v.role,Role::Candidate) && v.self_saved && v.round==Some(v.state.term) && granted && term==v.state.term && context.sent.request.term==term && context.sent.request.log.term==v.state.log.term && context.sent.request.log.index==v.state.log.index && !self.has_vote(context.peer) {
                self.voters=with_vote(&self.voters,context.peer);return Collect::Counted;
            }
        }
        Collect::Ignored
    }

    // No external count, boolean or certificate: inspect this Core's own votes.
    fn try_become_leader(&mut self) -> (elected: bool)
        requires old(self).wf(),
        ensures self.wf(),
            elected == promotion_allowed(old(self).view(),old(self).votes()),
            self.view() == promotion_after(old(self).view(),old(self).votes()),
            self.votes() == old(self).votes(), self.choices() == old(self).choices(),
            self.base() == old(self).base(),
        no_unwind
    {
        proof { assert(self.wf()); }
        let v=self.state;
        if !v.halted && v.pending.is_none() && matches!(v.role,Role::Candidate) && v.self_saved && v.round==Some(v.state.term) && v.state.voted_for==Some(v.config.node) && self.has_vote(v.config.node) && self.vote_count()>v.config.members/2 {
            self.state.role=Role::Leader;true
        } else {false}
    }

    fn vote_count(&self) -> (out: u64) requires self.wf(), ensures out as nat == self.votes().len()
    {
        proof { assert(self.wf()); self.voters@.unique_seq_to_set(); }
        self.voters.len() as u64
    }

    fn has_vote(&self, member: u64) -> (out: bool) requires self.wf(), ensures out == self.votes().contains(member)
    {
        let mut i=0;
        while i<self.voters.len()
            invariant i<=self.voters.len(),forall|j:int| 0<=j<i ==> self.voters@[j]!=member,
            decreases self.voters.len()-i,
        { if self.voters[i]==member {return true;} i+=1; }
        false
    }

    fn demote(&mut self, term:u64)
        requires old(self).wf(), old(self).state.pending.is_none(), term>=old(self).state.state.term,
        ensures self.wf(),
            self.state == (StateView{state:Persistent{term,voted_for:None,log:old(self).state.state.log},role:Role::Follower,round:None,self_saved:false,..old(self).state}),
            self.votes()==Set::<u64>::empty(),self.choices()==old(self).choices(),self.base()==old(self).base(),
    {
        self.state.state.term=term;self.state.state.voted_for=None;self.state.role=Role::Follower;
        self.state.round=None;self.state.self_saved=false;self.voters=Vec::new();
    }
}
fn with_vote(v:&Vec<u64>,n:u64) -> (out:Vec<u64>)
    requires v.len()<u64::MAX, !v@.contains(n),v@.no_duplicates(),
    ensures out@==v@.push(n),out@.to_set()==v@.to_set().insert(n),out@.no_duplicates(),
{
    let mut out=Vec::new();let mut i=0;
    while i<v.len()
        invariant i<=v.len(),out@==v@.subrange(0,i as int),
        decreases v.len()-i,
    {out.push(v[i]);i+=1;}
    assert(out@==v@);out.push(n);
    assert(out@.to_set() =~= v@.to_set().insert(n));out
}
fn state_valid(c:Config,s:Persistent) -> (ok:bool) ensures ok==valid_state(c,s) {
    c.members>0 && c.node<c.members && ((s.log.index==0)==(s.log.term==0)) && s.log.term<=s.term && match s.voted_for {Some(n)=>s.term>0 && n<c.members,None=>true}
}
fn request_valid(c:Config,r:Request) -> (ok:bool) ensures ok==valid_request(c,r) {
    r.term>0 && r.candidate<c.members && ((r.log.index==0)==(r.log.term==0)) && r.log.term<=r.term
}
fn context_valid(v:StateView,c:RpcContext) -> (ok:bool) ensures ok==valid_context(v,c) {
    c.sent.config.node==v.config.node && c.sent.config.members==v.config.members && c.peer<v.config.members && c.peer!=v.config.node && c.sent.request.candidate==v.config.node && request_valid(v.config,c.sent.request)
}
}
