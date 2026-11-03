//! Fixed mechanical environment: real candidate calls, retained writes and instance-bound callbacks.
//! No Raft invariant, choice uniqueness or majority assumption is supplied here.
use crate::{vote_api::*, vote_core::Core, vote_network as old, targets};
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy)]
pub enum Step { Start(usize), Promote(usize), Receive(old::Dispatch), Write{node:usize,success:bool}, Notify(usize), Deliver(usize), Error(old::Dispatch), Stop(usize), Restart(usize) }
#[derive(Clone, Copy)]
pub struct Io { pub proposed:old::Proposed, pub epoch:u64, pub result:Option<bool> }
#[derive(Clone, Copy)]
pub struct Image { pub record:Record, pub epoch:u64, pub at:usize }
pub struct Frame {
    pub step:Step, pub node:usize, pub epoch:u64,
    pub before:old::Observation, pub after:old::Observation,
    pub was_live:bool, pub is_live:bool, pub io:Option<Io>, pub image:Option<Image>,
}
pub struct Trace {
    pub net:old::Trace, pub live:Seq<bool>, pub epochs:Seq<u64>, pub io:Seq<Option<Io>>,
    pub images:Seq<Option<Image>>, pub ready:Seq<bool>, pub send_epochs:Seq<u64>,
    pub choices:Seq<Choice>, pub frames:Seq<Frame>,
}
pub struct Network {
    pub tick:usize, pub cores:Vec<Option<Core>>, pub io:Vec<Option<Io>>, pub images:Vec<Option<Image>>,
    pub ready:Vec<bool>, pub epochs:Vec<u64>, pub sent:Vec<old::Sent>,
    pub replies:Vec<old::Returned>, pub send_epochs:Vec<u64>, pub history:Ghost<Trace>,
}
impl Network {
    pub open spec fn trace(&self) -> Trace { self.history@ }
    // Representation bookkeeping only. Offline observations are the last actual
    // state, never a usable old Core. The Core itself has been dropped.
    pub open spec fn represented(&self) -> bool {
        let t=self.trace();
        &&& shape(t) && self.tick==t.frames.len() && self.cores.len()==t.net.nodes.len()
        &&& self.io@==t.io && self.images@==t.images && self.ready@==t.ready && self.epochs@==t.epochs
        &&& self.sent@==t.net.sent && self.replies@==t.net.replies && self.send_epochs@==t.send_epochs
        &&& forall|i:int| 0<=i<self.cores.len() ==> {
            &&& (#[trigger] self.cores@[i]).is_some()==t.live[i]
            &&& (self.cores@[i].is_some() ==> old::observe(self.cores@[i].unwrap())==t.net.nodes[i])
        }
    }
}
pub open spec fn shape(t:Trace) -> bool {
    let n=t.net.nodes.len();
    &&& old::shape(t.net) && n<=u64::MAX
    &&& t.live.len()==n && t.epochs.len()==n && t.io.len()==n && t.images.len()==n && t.ready.len()==n
    &&& t.send_epochs.len()==t.net.sent.len()
    &&& forall|i:int| 0<=i<n ==> {
        let p=#[trigger] t.io[i];
        &&& t.net.pending[i]==(match p {Some(p)=>Some(p.proposed),None=>None})
        &&& (p.is_some() ==> t.live[i] && p.unwrap().epoch==t.epochs[i])
    }
}
pub open spec fn boot(t:Trace) -> bool {
    shape(t) && old::boot(t.net) && t.choices==Seq::<Choice>::empty() && t.frames==Seq::<Frame>::empty()
        && forall|i:int| 0<=i<t.net.nodes.len() ==> t.live[i] && t.epochs[i]==0
            && t.io[i].is_none() && t.images[i].is_none() && !t.ready[i]
}
pub open spec fn routed(t:Trace,d:old::Dispatch) -> bool { old::routed(t.net,d) }
pub open spec fn callback(t:Trace,d:old::Dispatch) -> bool {
    routed(t,d) && t.live[t.net.sent[d.send as int].node as int]
        && t.send_epochs[d.send as int]==t.epochs[t.net.sent[d.send as int].node as int]
}
pub open spec fn window(t:Trace,n:int) -> bool {
    t.images[n].is_some() && (t.ready[n] || (t.io[n].is_some() && t.io[n].unwrap().result==Some(true)))
}
pub open spec fn enabled(t:Trace,s:Step) -> bool {
    match s {
        Step::Start(n)|Step::Promote(n) => n<t.live.len() && t.live[n as int],
        Step::Receive(d) => routed(t,d) && t.live[d.peer as int],
        Step::Error(d) => callback(t,d),
        Step::Deliver(i) => i<t.net.replies.len() && t.net.replies[i as int].request.is_some()
            && callback(t,t.net.replies[i as int].request.unwrap()),
        Step::Write{node:n,success:_} => n<t.live.len() && t.live[n as int] && t.io[n as int].is_some() && t.io[n as int].unwrap().result.is_none(),
        Step::Notify(n) => n<t.live.len() && t.live[n as int] && t.io[n as int].is_some() && t.io[n as int].unwrap().result.is_some(),
        Step::Stop(n) => n<t.live.len() && t.live[n as int] && window(t,n as int),
        Step::Restart(n) => n<t.live.len() && !t.live[n as int] && t.images[n as int].is_some() && t.epochs[n as int]<u64::MAX,
    }
}
pub open spec fn actor(t:Trace,s:Step) -> int {
    match s {
        Step::Start(n)|Step::Promote(n)|Step::Notify(n)|Step::Stop(n)|Step::Restart(n)|Step::Write{node:n,success:_} => n as int,
        Step::Receive(d) => d.peer as int,
        Step::Error(d) => t.net.sent[d.send as int].node as int,
        Step::Deliver(i) => t.net.sent[t.net.replies[i as int].request.unwrap().send as int].node as int,
    }
}
pub open spec fn call_step(t:Trace,s:Step) -> old::Step {
    match s {
        Step::Start(n)=>old::Step::Start(n), Step::Promote(n)=>old::Step::Promote(n),
        Step::Receive(d)=>old::Step::Receive(d), Step::Error(d)=>old::Step::Error(d),
        Step::Deliver(i)=>old::Step::Deliver(i),
        Step::Notify(n)=>old::Step::Save{node:n,success:t.io[n as int].unwrap().result.unwrap()},
        _=>old::Step::Start(0), // Never used for I/O, Stop or Restart.
    }
}
pub open spec fn proposal(t:Trace,s:Step) -> Option<old::Proposed> {
    let a=actor(t,s);let o=t.net.nodes[a];
    match s {
        Step::Start(_)=>match start_result(o.state) {Campaign::Save(w)=>Some(old::Proposed{write:w,request:None}),_=>None},
        Step::Receive(d)=>match receive_result(o.state,t.net.sent[d.send as int].payload.request) {Receive::Save(w)=>Some(old::Proposed{write:w,request:Some(d)}),_=>None},
        Step::Deliver(i)=>match collect_result(o.state,o.votes,old::context(t.net,t.net.replies[i as int].request.unwrap()),old::reply_result(t.net.replies[i as int])) {Collect::Save(w)=>Some(old::Proposed{write:w,request:None}),_=>None},
        Step::Error(d)=>match collect_result(o.state,o.votes,old::context(t.net,d),RpcResult::Error) {Collect::Save(w)=>Some(old::Proposed{write:w,request:None}),_=>None},_=>None,
    }
}
pub open spec fn protocol(t:Trace,s:Step) -> Trace {
    let a=actor(t,s);let o=t.net.nodes[a];
    match s {
        Step::Write{node,success} => {
            let p=t.io[a].unwrap();
            Trace{io:t.io.update(a,Some(Io{result:Some(success),..p})),ready:t.ready.update(a,false),
                images:if success {t.images.update(a,Some(Image{record:p.proposed.write.record,epoch:p.epoch,at:t.frames.len() as usize}))} else {t.images},..t}
        },
        Step::Stop(_) => Trace{live:t.live.update(a,false),io:t.io.update(a,None),ready:t.ready.update(a,false),
            net:old::Trace{pending:t.net.pending.update(a,None),..t.net},..t},
        Step::Restart(_) => {
            let r=t.images[a].unwrap().record;let c=o.state.config;
            let ok=valid_record(c,r);
            Trace{live:t.live.update(a,ok),epochs:t.epochs.update(a,(t.epochs[a]+1) as u64),ready:t.ready.update(a,false),
                net:old::Trace{nodes:if ok {t.net.nodes.update(a,old::Observation{state:recovered_view(c,r),votes:Set::empty(),choices:Seq::empty(),base:Some(r)})} else {t.net.nodes},..t.net},..t}
        },
        _ => {
            let u=old::protocol_after(t.net,call_step(t,s));
            let proposed=proposal(t,s);
            let p=match s {Step::Notify(_)=>None,
                _=>if proposed.is_some() {Some(Io{proposed:proposed.unwrap(),epoch:t.epochs[a],result:None})} else {t.io[a]}};
            let choices=match s {
                Step::Start(_) => start_choices(t.choices,o.state,start_result(o.state)),
                Step::Notify(_) => append_action(t.choices,complete_result(o.state,Completion{op:t.io[a].unwrap().proposed.write.record.op,success:t.io[a].unwrap().result.unwrap()})),
                _ => t.choices,
            };
            let ready=match s {Step::Notify(_)=>t.io[a].unwrap().result==Some(true),
                _=>t.ready[a] && u.nodes[a].state.state==o.state.state && u.nodes[a].state.next==o.state.next && proposed.is_none()};
            Trace{net:u,io:t.io.update(a,p),ready:t.ready.update(a,ready),choices,
                send_epochs:if u.sent.len()>t.net.sent.len() {t.send_epochs.push(t.epochs[a])} else {t.send_epochs},..t}
        },
    }
}
pub open spec fn step_after(t:Trace,s:Step) -> Trace {
    let u=protocol(t,s);let a=actor(t,s);
    Trace{frames:t.frames.push(Frame{step:s,node:a as usize,epoch:t.epochs[a],before:t.net.nodes[a],after:u.net.nodes[a],
        was_live:t.live[a],is_live:u.live[a],io:t.io[a],image:t.images[a]}),..u}
}
pub open spec fn elects(f:Frame) -> bool {
    f.is_live && f.after.state.role==Role::Leader && (!f.was_live || f.before.state.role!=Role::Leader || f.before.state.state.term!=f.after.state.state.term)
}
pub open spec fn event(f:Frame,g:targets::Group) -> targets::Event {
    if elects(f) {targets::Event::Elect{who:targets::Key{group:g,node:f.node as nat},term:f.after.state.state.term as nat}} else {targets::Event::Silent}
}
pub open spec fn projection(t:Trace,g:targets::Group) -> Seq<targets::Event> {t.frames.map_values(|f:Frame|event(f,g))}
pub open spec fn historical_safety(t:Trace) -> bool {
    forall|i:int,j:int| #![trigger t.frames[i],t.frames[j]] 0<=i<t.frames.len() && 0<=j<t.frames.len()
        && elects(t.frames[i]) && elects(t.frames[j]) && t.frames[i].after.state.state.term==t.frames[j].after.state.state.term
        ==> t.frames[i].node==t.frames[j].node
}
pub open spec fn choice_for(t:Trace,v:u64,n:u64,term:u64) -> bool {t.choices.contains(Choice{node:v,term,candidate:n})}
pub open spec fn evidence(t:Trace,n:int,v:u64,term:u64) -> bool {
    choice_for(t,v,n as u64,term) && if v==n {old::self_evidence(t.net,n,term)} else {old::remote_evidence(t.net,n,term,v)}
}
pub open spec fn provenance(t:Trace) -> bool {
    &&& forall|a:int,v:u64| 0<=a<t.net.nodes.len() && t.live[a] && #[trigger] t.net.nodes[a].votes.contains(v)
        ==> evidence(t,a,v,t.net.nodes[a].state.state.term)
    &&& forall|i:int| 0<=i<t.frames.len() && elects(#[trigger] t.frames[i]) ==> {
        let f=t.frames[i];
        &&& f.was_live && promotion_allowed(f.before.state,f.before.votes)
        &&& f.after.state==promotion_after(f.before.state,f.before.votes)
        &&& forall|v:u64| #[trigger] f.before.votes.contains(v) ==> evidence(t,f.node as int,v,f.after.state.state.term)
    }
}
pub open spec fn image_source(t:Trace,i:Image,n:int) -> bool {
    i.at<t.frames.len() && t.frames[i.at as int].node==n && t.frames[i.at as int].epoch==i.epoch
        && t.frames[i.at as int].step==Step::Write{node:n as usize,success:true}
        && t.frames[i.at as int].io.is_some() && t.frames[i.at as int].io.unwrap().proposed.write.record==i.record
}
pub open spec fn recovered_from_own_write(t:Trace) -> bool {
    &&& forall|n:int| 0<=n<t.images.len() && (#[trigger] t.images[n]).is_some() ==> image_source(t,t.images[n].unwrap(),n)
    &&& forall|k:int| 0<=k<t.frames.len() && (#[trigger] t.frames[k]).step is Restart ==> {
        let f=t.frames[k];
        &&& !f.was_live && f.is_live && f.image.is_some() && image_source(t,f.image.unwrap(),f.node as int)
        &&& f.after.state==recovered_view(f.before.state.config,f.image.unwrap().record)
        &&& f.after.base==Some(f.image.unwrap().record) && f.after.choices==Seq::<Choice>::empty() && f.after.votes==Set::<u64>::empty()
    }
}
pub proof fn bridge(t:Trace,g:targets::Group,h:targets::History)
    requires historical_safety(t), h.events==projection(t,g), ensures targets::election_safety(h),
{
    assert forall|i:int,j:int| #![trigger h.events[i],h.events[j]] 0<=i<h.events.len() && 0<=j<h.events.len() implies
        match (h.events[i],h.events[j]) {
            (targets::Event::Elect{who:a,term:x},targets::Event::Elect{who:b,term:y}) => a.group==b.group && x==y ==> a.node==b.node,
            _ => true,
        } by {assert(h.events[i]==event(t.frames[i],g));assert(h.events[j]==event(t.frames[j],g));}
}
pub fn new_network(initials:&Vec<Initial>) -> (n:Network)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
    ensures n.represented(),boot(n.trace()),n.cores.len()==initials.len(),
        forall|i:int| 0<=i<initials.len() ==> (#[trigger] n.trace().net.nodes[i]).state==fresh_view(Config{node:i as u64,members:initials.len() as u64},initials[i]),
{
    let mut cores:Vec<Option<Core>>=Vec::new();let mut io=Vec::new();let mut images=Vec::new();let mut ready=Vec::new();let mut epochs=Vec::new();
    let ghost mut obs=Seq::<old::Observation>::empty();let mut i:usize=0;
    while i<initials.len()
        invariant i<=initials.len(),0<initials.len()<=u64::MAX,
            forall|j:int| 0<=j<initials.len() ==> valid_initial(Config{node:j as u64,members:initials.len() as u64},#[trigger] initials[j]) && initials[j].role==Role::Follower,
            cores.len()==i,obs.len()==i,io@==Seq::new(i as nat,|unused:int|None::<Io>),images@==Seq::new(i as nat,|unused:int|None::<Image>),
            ready@==Seq::new(i as nat,|unused:int|false),epochs@==Seq::new(i as nat,|unused:int|0u64),
            forall|j:int| 0<=j<i ==> (#[trigger] cores@[j]).is_some() && old::observe(cores@[j].unwrap())==obs[j]
                && obs[j]==(old::Observation{state:fresh_view(Config{node:j as u64,members:initials.len() as u64},initials[j]),votes:Set::empty(),choices:Seq::empty(),base:None}),
        decreases initials.len()-i,
    {
        let c=Core::new(Config{node:i as u64,members:initials.len() as u64},initials[i]).unwrap();
        proof{obs=obs.push(old::observe(c));}
        cores.push(Some(c));io.push(None);images.push(None);ready.push(false);epochs.push(0);i+=1;
    }
    let ghost net=old::Trace{nodes:obs,pending:Seq::new(i as nat,|unused:int|None::<old::Proposed>),sent:Seq::empty(),replies:Seq::empty(),delivered:Seq::empty(),frames:Seq::empty()};
    let ghost history=Trace{net,live:Seq::new(i as nat,|unused:int|true),epochs:epochs@,io:io@,images:images@,ready:ready@,send_epochs:Seq::empty(),choices:Seq::empty(),frames:Seq::empty()};
    assert forall|j:int| 0<=j<i implies {
        &&& obs[j].state==fresh_view(Config{node:j as u64,members:i as u64},initials[j])
        &&& obs[j].state.config==Config{node:j as u64,members:i as u64}
        &&& valid_state(obs[j].state.config,obs[j].state.state) && obs[j].state.role==Role::Follower
        &&& obs[j].state.next==0 && obs[j].state.pending.is_none() && !obs[j].state.halted
        &&& obs[j].state.round.is_none() && !obs[j].state.self_saved
        &&& obs[j].votes==Set::<u64>::empty() && obs[j].choices==Seq::<Choice>::empty() && obs[j].base.is_none()
    } by {assert(cores@[j].is_some());assert(old::observe(cores@[j].unwrap())==obs[j]);assert(obs[j]==(old::Observation{state:fresh_view(Config{node:j as u64,members:i as u64},initials[j]),votes:Set::empty(),choices:Seq::empty(),base:None}));}
    assert(shape(history));assert(old::boot(net));assert(boot(history));
    Network{tick:0,cores,io,images,ready,epochs,sent:Vec::new(),replies:Vec::new(),send_epochs:Vec::new(),history:Ghost(history)}
}
pub fn equal_persistent(a:Persistent,b:Persistent) -> (yes:bool) ensures yes==(a==b) {
    a.term==b.term && a.voted_for==b.voted_for && a.log.term==b.log.term && a.log.index==b.log.index
}
// Fixed source correspondence; the obligations above are NOT preconditions.
pub fn execute(n:&mut Network,s:Step)
    requires old(n).represented(),enabled(old(n).trace(),s),old(n).trace().frames.len()<usize::MAX,
    ensures n.represented(),n.trace()==step_after(old(n).trace(),s),
        n.trace().frames.drop_last()==old(n).trace().frames,n.cores.len()==old(n).cores.len(),
{
    let ghost before=n.trace();let ghost expected=step_after(before,s);let ghost cores_before=n.cores@;
    let at=match s {
        Step::Start(i)|Step::Promote(i)|Step::Notify(i)|Step::Stop(i)|Step::Restart(i)|Step::Write{node:i,success:_}=>i,
        Step::Receive(d)=>d.peer as usize,Step::Error(d)=>n.sent[d.send].node as usize,
        Step::Deliver(i)=>n.sent[n.replies[i].request.unwrap().send].node as usize,
    };
    let epoch=n.epochs[at];let mut slot=n.cores.remove(at);
    let ghost mut ledger=before.choices;
    match s {
        Step::Stop(_) => {slot=None;n.io[at]=None;n.ready[at]=false;},
        Step::Restart(_) => {
            let image=n.images[at].unwrap();
            slot=Core::recover(Config{node:at as u64,members:(n.cores.len()+1) as u64},image.record);
            n.epochs[at]=epoch+1;n.ready[at]=false;
        },
        Step::Write{success,..} => {
            let p=n.io[at].unwrap();n.io[at]=Some(Io{result:Some(success),..p});n.ready[at]=false;
            if success {n.images[at]=Some(Image{record:p.proposed.write.record,epoch,at:n.tick});}
        },
        _ => {
            let mut c=slot.unwrap();let prior=c.inspect();let mut issued=false;
            match s {
                Step::Start(_) => {
                    let out=c.start_election();
                    if let Campaign::Save(w)=out {
                        n.io[at]=Some(Io{proposed:old::Proposed{write:w,request:None},epoch,result:None});issued=true;
                        let actual=c.inspect();proof{ledger=ledger.push(choice(actual));}
                    }
                },
                Step::Promote(_) => {c.try_become_leader();},
                Step::Receive(d) => {
                    let out=c.receive(n.sent[d.send].payload.request);
                    if let Receive::Save(w)=out {n.io[at]=Some(Io{proposed:old::Proposed{write:w,request:Some(d)},epoch,result:None});issued=true;}
                },
                Step::Notify(_) => {
                    let p=n.io[at].unwrap();let notice=Completion{op:p.proposed.write.record.op,success:p.result.unwrap()};
                    let action=c.complete(notice);n.io[at]=None;
                    proof{ledger=append_action(ledger,action);}
                    match action {
                        Some(Action::Send(payload))=>{n.sent.push(old::Sent{node:at as u64,write:p.proposed.write,notice,payload});n.send_epochs.push(epoch);},
                        Some(Action::Reply(payload))=>{n.replies.push(old::Returned{node:at as u64,request:p.proposed.request,payload});},
                        _=>{},
                    }
                    n.ready[at]=notice.success;
                },
                Step::Deliver(i) => {
                    let r=n.replies[i];let d=r.request.unwrap();
                    let out=c.collect(RpcContext{sent:n.sent[d.send].payload,peer:d.peer},RpcResult::Reply{term:r.payload.term,granted:r.payload.granted});
                    if let Collect::Save(w)=out {n.io[at]=Some(Io{proposed:old::Proposed{write:w,request:None},epoch,result:None});issued=true;}
                },
                Step::Error(d) => {
                    let out=c.collect(RpcContext{sent:n.sent[d.send].payload,peer:d.peer},RpcResult::Error);
                    if let Collect::Save(w)=out {n.io[at]=Some(Io{proposed:old::Proposed{write:w,request:None},epoch,result:None});issued=true;}
                }, _=>{},
            }
            let actual=c.inspect();
            if !matches!(s,Step::Notify(_)) && (issued || !equal_persistent(prior.state,actual.state) || prior.next!=actual.next) {n.ready[at]=false;}
            assert(old::observe(c)==expected.net.nodes[at as int]);slot=Some(c);
        },
    }
    if let Some(c)=&slot {let actual=c.inspect();assert(actual==expected.net.nodes[at as int].state);}
    assert(slot.is_some()==expected.live[at as int]);
    assert(slot.is_some() ==> old::observe(slot.unwrap())==expected.net.nodes[at as int]);
    assert(ledger==expected.choices);
    n.cores.insert(at,slot);
    assert forall|j:int| 0<=j<n.cores.len() && j!=at implies #[trigger] n.cores@[j]==cores_before[j] by {}
    assert(n.io@==expected.io);assert(n.images@==expected.images);assert(n.ready@==expected.ready);
    assert(n.epochs@==expected.epochs);assert(n.sent@==expected.net.sent);assert(n.replies@==expected.net.replies);assert(n.send_epochs@==expected.send_epochs);
    assert forall|j:int| 0<=j<n.cores.len() implies {
        &&& (#[trigger] n.cores@[j]).is_some()==expected.live[j]
        &&& (n.cores@[j].is_some() ==> old::observe(n.cores@[j].unwrap())==expected.net.nodes[j])
        &&& expected.net.nodes[j].state.config==Config{node:j as u64,members:n.cores.len() as u64}
    } by {if j!=at {assert(n.cores@[j]==cores_before[j]);assert(expected.net.nodes[j]==before.net.nodes[j]);}}
    assert forall|j:int| 0<=j<expected.net.sent.len() implies (#[trigger] expected.net.sent[j]).node<expected.net.nodes.len() by {
        if j<before.net.sent.len() {assert(expected.net.sent[j]==before.net.sent[j]);}
    }
    assert forall|j:int| 0<=j<expected.io.len() implies {
        let p=#[trigger] expected.io[j];
        &&& expected.net.pending[j]==(match p {Some(p)=>Some(p.proposed),None=>None})
        &&& (p.is_some() ==> expected.live[j] && p.unwrap().epoch==expected.epochs[j])
    } by {if j!=at {assert(expected.io[j]==before.io[j]);} }
    assert(shape(expected));
    n.tick=n.tick+1;proof{n.history@=expected;}
}
// Structural routing, lifecycle and incarnation checks only.
pub fn is_enabled(n:&Network,s:Step) -> (yes:bool)
    requires n.represented(), ensures yes==enabled(n.trace(),s),
{
    match s {
        Step::Start(i)|Step::Promote(i)=>i<n.cores.len() && n.cores[i].is_some(),
        Step::Receive(d)=>d.send<n.sent.len() && d.peer<n.cores.len() as u64 && d.peer!=n.sent[d.send].node && n.cores[d.peer as usize].is_some(),
        Step::Error(d)=>callback_enabled(n,d),
        Step::Deliver(i)=>if i<n.replies.len() {if let Some(d)=n.replies[i].request {callback_enabled(n,d)} else {false}} else {false},
        Step::Write{node:i,success:_}=>i<n.cores.len() && n.cores[i].is_some() && n.io[i].is_some() && n.io[i].unwrap().result.is_none(),
        Step::Notify(i)=>i<n.cores.len() && n.cores[i].is_some() && n.io[i].is_some() && n.io[i].unwrap().result.is_some(),
        Step::Stop(i)=>i<n.cores.len() && n.cores[i].is_some() && n.images[i].is_some() && (n.ready[i] || (n.io[i].is_some() && n.io[i].unwrap().result==Some(true))),
        Step::Restart(i)=>i<n.cores.len() && n.cores[i].is_none() && n.images[i].is_some() && n.epochs[i]<u64::MAX,
    }
}
pub fn callback_enabled(n:&Network,d:old::Dispatch) -> (yes:bool)
    requires n.represented(), ensures yes==callback(n.trace(),d),
{
    if d.send>=n.sent.len() || d.peer>=n.cores.len() as u64 || d.peer==n.sent[d.send].node {return false;}
    let a=n.sent[d.send].node as usize;
    n.cores[a].is_some() && n.send_epochs[d.send]==n.epochs[a]
}
}
