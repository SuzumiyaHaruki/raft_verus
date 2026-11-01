//! Deliberately wrong maintenance functions. Never supplied to an experiment.
#[path="api.rs"] mod vote_api;
use vote_api::*;
use vote_api::View;
use vstd::prelude::*;
verus! {
pub open spec fn sample() -> View {
    fresh_view(Config{node:0,members:3},Initial{state:Persistent{term:3,voted_for:Some(1),
        log:Log{term:2,index:10}},role:Role::Follower})
}
#[cfg(any(bad_term,bad_candidate,bad_request_term,bad_log))]
pub fn wrong_start(v: View) -> (out: Campaign)
    requires v == sample(),
    ensures out == start_result(v),
{
    // Hard-coded defective examples, not a general campaign implementation.
    let term = if cfg!(bad_term) { 3 } else { 4 };
    let candidate = if cfg!(bad_candidate) { 1 } else { 0 };
    let request_term = if cfg!(bad_request_term) { 9 } else { term };
    let log = if cfg!(bad_log) { Log{term:0,index:0} } else { v.state.log };
    let request = Request{term:request_term,candidate,log};
    Campaign::Save(Write{record:Record{op:0,state:Persistent{term,voted_for:Some(candidate),log:v.state.log}},
        request,action:Action::Send(Send{config:v.config,request})})
}
#[cfg(bad_split)]
pub fn shadow_self_choice(receiver: View) -> (out: (View, Choice))
    requires receiver == sample(),
    ensures out.0 == receiver, out.1 == (Choice{node:0,term:3,candidate:0}),
{ (receiver, Choice{node:0,term:3,candidate:0}) }
#[cfg(bad_split)]
pub fn actual_receiver_reply(receiver: View) -> (out: Reply)
    requires receiver == sample(),
    ensures out == (Reply{node:0,term:3,candidate:1,granted:true}),
{ Reply{node:0,term:receiver.state.term,candidate:1,granted:true} }
#[cfg(bad_split)]
pub fn fixed_mixed_observer(v: View)
    requires v == sample(),
{
    let (receiver, own) = shadow_self_choice(v);
    let reply = actual_receiver_reply(receiver);
    assert(unique(seq![own,Choice{node:reply.node,term:reply.term,candidate:reply.candidate}]));
}
#[cfg(bad_early)]
pub fn send_without_completion(v: View,n: Completion) -> (out: Option<Action>)
    requires v == sample(),
    ensures out == complete_result(v,n),
{
    Some(Action::Send(Send{config:v.config,request:Request{term:4,candidate:0,log:v.state.log}}))
}
#[cfg(any(bad_recovery_vote,bad_recovery_next))]
pub fn wrong_recover(c: Config,r: Record) -> (out: View)
    requires valid_record(c,r),
    ensures out == recovered_view(c,r),
{
    let next = if cfg!(bad_recovery_next) { r.op } else { r.op + 1 };
    let vote = if cfg!(bad_recovery_vote) { None } else { r.state.voted_for };
    View{config:c,state:Persistent{term:r.state.term,voted_for:vote,log:r.state.log},role:Role::Follower,
        next,pending:None,halted:false}
}
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> View {View{config:Config{node:0,members:3},state:Persistent{term:3,voted_for:Some(1),
        log:Log{term:2,index:10}},role:Role::Follower,next:0,pending:None,halted:false}}
    #[test] fn semantic_rejection() {
        let v=input();
        #[cfg(any(bad_term,bad_candidate,bad_request_term,bad_log))] {
            let Campaign::Save(w)=wrong_start(v) else {panic!("expected concrete defective save")};
            #[cfg(bad_term)] assert_eq!(w.record.state.term,4,"must enter next term");
            #[cfg(bad_candidate)] assert_eq!(w.record.state.voted_for,Some(0),"must choose self");
            #[cfg(bad_request_term)] assert_eq!(w.request.term,4,"request must belong to new round");
            #[cfg(bad_log)] assert_eq!(w.request.log,v.state.log,"request must keep actual log");
        }
        #[cfg(bad_split)] {
            let (receiver,a)=shadow_self_choice(v);let b=actual_receiver_reply(receiver);
            assert!(a.term!=b.term || a.candidate==b.candidate,"mixed history must include self vote");
        }
        #[cfg(bad_early)] assert_eq!(send_without_completion(v,Completion{op:9,success:false}),None,"no unmatched send authorization");
        #[cfg(any(bad_recovery_vote,bad_recovery_next))] {
            let r=Record{op:2,state:Persistent{term:4,voted_for:Some(0),log:v.state.log}};
            let out=wrong_recover(v.config,r);
            #[cfg(bad_recovery_vote)] assert_eq!(out.state.voted_for,Some(0),"restore retained self vote");
            #[cfg(bad_recovery_next)] assert_eq!(out.next,3,"do not reuse retained operation ID");
        }
    }
}
