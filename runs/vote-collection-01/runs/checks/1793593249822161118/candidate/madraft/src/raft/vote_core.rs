//! Executable receiver, campaign, and reply collection share this state.
use super::vote_api::*;
use super::vote_api::View as StateView;
use vstd::prelude::*;
verus! {
broadcast use {vstd::seq_lib::group_seq_properties, vstd::set::group_set_axioms, vstd::set_lib::group_set_properties};

struct Data { state: StateView, outputs: Ghost<Seq<Choice>>, origin: Ghost<Option<Record>>, voters: Vec<u64> }
impl Data {
    pub closed spec fn view(&self) -> StateView { self.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.origin@ }
    pub closed spec fn votes(&self) -> Set<u64> { self.voters@.to_set() }

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
            !self.state.halted && w.record.state==self.state.state && w.record.op<self.state.next
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

}
pub struct Core { data: Data }
impl Core {
    pub closed spec fn view(&self) -> StateView { self.data.state }
    pub closed spec fn choices(&self) -> Seq<Choice> { self.data.outputs@ }
    pub closed spec fn base(&self) -> Option<Record> { self.data.origin@ }
    pub closed spec fn votes(&self) -> Set<u64> { self.data.voters@.to_set() }
    #[verifier::type_invariant]
    closed spec fn wf(&self) -> bool { self.data.wf() }
    fn take_data(&mut self) -> (out:Data)
        ensures out==old(self).data, out.wf(),
    {
        proof { use_type_invariant(&*self); }
        let mut data=Data{state:StateView{role:Role::Follower,pending:None,round:None,self_saved:false,..self.data.state},
            outputs:Ghost(Seq::empty()),origin:Ghost(None),voters:Vec::new()};
        proof { empty_facts(); }
        core::mem::swap(&mut self.data,&mut data);
        data
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
        proof { empty_facts(); }
        Some(Core{data:Data{state:StateView{config:c,state:initial.state,role:initial.role,next:0,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(None),voters:Vec::new()}})
    }
    pub fn recover(c: Config, record: Record) -> (out: Option<Self>)
        ensures out.is_some() <==> valid_record(c,record),
            out.is_some() ==> out.unwrap().view() == recovered_view(c,record)
                && out.unwrap().choices() == Seq::<Choice>::empty() && out.unwrap().base() == Some(record) && out.unwrap().votes() == Set::<u64>::empty(),
    {
        if !valid(c,record.state) || record.op==u64::MAX { return None; }
        proof { empty_facts(); }
        Some(Core{data:Data{state:StateView{config:c,state:record.state,role:Role::Follower,next:record.op+1,pending:None,halted:false,round:None,self_saved:false},outputs:Ghost(Seq::empty()),origin:Ghost(Some(record)),voters:Vec::new()}})
    }
    pub fn inspect(&self) -> (out: StateView) ensures out == self.view() { self.data.state }

    pub fn receive(&mut self, request: Request) -> (out: Receive)
        ensures out == receive_result(old(self).view(), request),
            self.view() == receive_after(old(self).view(),request,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
            self.votes() == receive_votes(old(self).view(),old(self).votes(),request,out),
    {
        proof { use_type_invariant(&*self); }
        let mut data=self.take_data();
        proof { empty_facts(); }
        let ghost prior=data;
        if data.state.halted { self.data=data; return Receive::Reject(Fault::Halted); }
        if data.state.pending.is_some() { self.data=data; return Receive::Reject(Fault::Busy); }
        if !request_valid(data.state.config,request) { self.data=data; return Receive::Reject(Fault::Invalid); }
        let higher=request.term>data.state.state.term;
        if data.state.next==u64::MAX {
            if higher { data.state.state.term=request.term;data.state.state.voted_for=None;data.state.role=Role::Follower;data.state.round=None;data.state.self_saved=false;data.voters=Vec::new(); data.state.halted=true; }
            self.data=data; return Receive::Reject(Fault::Exhausted);
        }
        let grant=request.term>=data.state.state.term
            && (request.log.term>data.state.state.log.term || (request.log.term==data.state.state.log.term && request.log.index>=data.state.state.log.index))
            && (higher || data.state.state.voted_for.is_none() || data.state.state.voted_for==Some(request.candidate));
        if higher { data.state.state.term=request.term;data.state.state.voted_for=None;data.state.role=Role::Follower;data.state.round=None;data.state.self_saved=false;data.voters=Vec::new(); }
        if grant { data.state.state.voted_for=Some(request.candidate); }
        let w=Write{record:Record{op:data.state.next,state:data.state.state},request:Some(request),action:Action::Reply(Reply{node:data.state.config.node,term:data.state.state.term,candidate:request.candidate,granted:grant})};
        data.state.next=data.state.next+1; data.state.pending=Some(w);
        proof { assert(data.wf()); }
        self.data=data; Receive::Save(w)
    }
    pub fn start_election(&mut self) -> (out: Campaign)
        ensures out == start_result(old(self).view()), self.view() == start_after(old(self).view(),out),
            self.choices() == start_choices(old(self).choices(),old(self).view(),out),
            self.base() == old(self).base(), self.votes() == start_votes(old(self).votes(),out),
    {
        proof { use_type_invariant(&*self); }
        let mut data=self.take_data();
        proof { empty_facts(); }
        let ghost prior=data;
        if data.state.halted { self.data=data; return Campaign::Reject(Fault::Halted); }
        if data.state.pending.is_some() { self.data=data; return Campaign::Reject(Fault::Busy); }
        if matches!(data.state.role,Role::Leader) { self.data=data; return Campaign::Reject(Fault::Leader); }
        if data.state.state.term==u64::MAX { self.data=data; return Campaign::Reject(Fault::TermExhausted); }
        if data.state.next==u64::MAX { self.data=data; return Campaign::Reject(Fault::Exhausted); }
        data.state.state.term=data.state.state.term+1;
        data.state.state.voted_for=Some(data.state.config.node);data.state.role=Role::Candidate;
        data.state.round=Some(data.state.state.term);data.state.self_saved=false;data.voters=Vec::new();
        let request=Request{term:data.state.state.term,candidate:data.state.config.node,log:data.state.state.log};
        let w=Write{record:Record{op:data.state.next,state:data.state.state},request:Some(request),action:Action::Send(Send{config:data.state.config,request})};
        data.state.next=data.state.next+1;data.state.pending=Some(w);
        proof { data.outputs@=data.outputs@.push(choice(data.state)); }
        proof { assert(data.wf()); }
        self.data=data; Campaign::Save(w)
    }
    pub fn complete(&mut self, notice: Completion) -> (out: Option<Action>)
        ensures out == complete_result(old(self).view(),notice),
            self.view() == complete_after(old(self).view(),notice),
            self.choices() == append_action(old(self).choices(),out), self.base() == old(self).base(),
            self.votes() == completion_votes(old(self).view(),old(self).votes(),notice),
    {
        proof { use_type_invariant(&*self); }
        let mut data=self.take_data();
        proof { empty_facts(); }
        let ghost prior=data;
        if let Some(w)=data.state.pending {
            if w.record.op==notice.op {
                data.state.pending=None;
                if !notice.success { data.state.halted=true;data.state.round=None;data.state.self_saved=false;data.voters=Vec::new();self.data=data; return None; }
                if let Action::Send(_)=w.action {
                    data.state.self_saved=true;let mut voters=Vec::new();voters.push(data.state.config.node);data.voters=voters;
                    proof { data.voters@.unique_seq_to_set(); }
                }
                proof { data.outputs@=append_action(data.outputs@,Some(w.action)); }
                proof { assert(data.wf()); }
                self.data=data; return Some(w.action);
            }
        }
        self.data=data; None
    }
    pub fn collect(&mut self, context: RpcContext, result: RpcResult) -> (out: Collect)
        ensures out == collect_result(old(self).view(),old(self).votes(),context,result),
            self.view() == collect_after(old(self).view(),context,result,out),
            self.votes() == collect_votes(old(self).view(),old(self).votes(),context,result,out),
            self.choices() == old(self).choices(), self.base() == old(self).base(),
    {
        proof { use_type_invariant(&*self); }
        let mut data=self.take_data();
        proof { empty_facts(); }
        let ghost prior=data;
        if data.state.halted { self.data=data; return Collect::Reject(Fault::Halted); }
        if data.state.pending.is_some() { self.data=data; return Collect::Reject(Fault::Busy); }
        if context.sent.config.node!=data.state.config.node || context.sent.config.members!=data.state.config.members
            || context.peer>=data.state.config.members || context.peer==data.state.config.node
            || context.sent.request.candidate!=data.state.config.node || !request_valid(data.state.config,context.sent.request) { self.data=data; return Collect::Reject(Fault::Invalid); }
        let out=match result {
            RpcResult::Error => Collect::Ignored,
            RpcResult::Reply{term,granted} => {
                if term>data.state.state.term {
                    data.state.state.term=term;data.state.state.voted_for=None;data.state.role=Role::Follower;data.state.round=None;data.state.self_saved=false;data.voters=Vec::new();
                    if data.state.next==u64::MAX { data.state.halted=true;self.data=data; return Collect::Reject(Fault::Exhausted); }
                    let w=Write{record:Record{op:data.state.next,state:data.state.state},request:None,action:Action::Persisted};
                    data.state.next=data.state.next+1;data.state.pending=Some(w);Collect::Save(w)
                } else if matches!(data.state.role,Role::Candidate) && data.state.self_saved && data.state.round==Some(data.state.state.term)
                    && granted && term==data.state.state.term && context.sent.request.term==term
                    && context.sent.request.log.term==data.state.state.log.term && context.sent.request.log.index==data.state.state.log.index && !contains(&data.voters,context.peer) {
                    data.voters.push(context.peer);
                    proof {
                        assert(data.votes()==prior.votes().insert(context.peer));
                        assert forall|n:u64| #[trigger] data.votes().contains(n) implies n<data.state.config.members by { }
                        bounded(data.votes(),data.state.config.members);
                    }
                    Collect::Counted
                } else { Collect::Ignored }
            }
        };
        proof { assert(data.wf()); }
        self.data=data; out
    }
    pub fn vote_count(&self) -> (out: u64) ensures out as nat == self.votes().len()
    { proof { use_type_invariant(self); self.data.voters@.unique_seq_to_set(); } self.data.voters.len() as u64 }
    pub fn has_vote(&self, member: u64) -> (out: bool) ensures out == self.votes().contains(member)
    {
        let mut i:usize=0;
        while i<self.data.voters.len()
            invariant i<=self.data.voters.len(), forall|j:int| 0<=j<i ==> self.data.voters@[j]!=member,
            decreases self.data.voters.len()-i,
        { if self.data.voters[i]==member { return true; } i+=1; }
        false
    }
}
proof fn bounded(s:Set<u64>,members:u64)
    requires s.finite(), forall|n:u64| #[trigger] s.contains(n) ==> n<members,
    ensures s.len()<=members,
    decreases members,
{
    if members==0 { assert(s =~= Set::empty()); }
    else {
        let last=(members-1) as u64;
        bounded(s.remove(last),last);
        if s.contains(last) { assert(s =~= s.remove(last).insert(last)); } else { assert(s.remove(last) =~= s); }
    }
}
proof fn empty_facts()
    ensures Seq::<u64>::empty().to_set()==Set::<u64>::empty(), Set::<u64>::empty().finite(), Set::<u64>::empty().len()==0,
{ broadcast use vstd::set::group_set_axioms; }
fn contains(voters:&Vec<u64>,member:u64) -> (out:bool)
    ensures out==voters@.to_set().contains(member),
{
    let mut i:usize=0;
    while i<voters.len()
        invariant i<=voters.len(), forall|j:int| 0<=j<i ==> voters@[j]!=member,
        decreases voters.len()-i,
    { if voters[i]==member { return true; } i+=1; }
    false
}
fn valid(c:Config,s:Persistent) -> (out:bool) ensures out==valid_state(c,s) {
    c.members>0 && c.node<c.members && (s.log.index==0)==(s.log.term==0) && s.log.term<=s.term
        && match s.voted_for {Some(n)=>s.term>0 && n<c.members,None=>true}
}
fn request_valid(c:Config,r:Request) -> (out:bool) ensures out==valid_request(c,r) {
    r.term>0 && r.candidate<c.members && (r.log.index==0)==(r.log.term==0) && r.log.term<=r.term
}
}
