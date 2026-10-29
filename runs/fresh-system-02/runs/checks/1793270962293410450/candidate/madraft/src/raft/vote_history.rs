//! Ghost history of actual vote decisions; recovery attachment remains OPEN.
use super::election::{self, Election, Vote, Reply};
use vstd::prelude::*;
verus! {
pub struct Machine { pub current:Election, pub ghost history:Map<u64,u64> }
pub open spec fn valid_vote(v:Machine) -> bool {
    (forall|term:u64| v.history.contains_key(term) ==> term<=v.current.term)
    && v.current.voted_for == (if v.history.contains_key(v.current.term) {Some(v.history[v.current.term])} else {None})
}
pub open spec fn extends(a:Map<u64,u64>,b:Map<u64,u64>) -> bool {
    forall|term:u64| a.contains_key(term) ==> b.contains_key(term) && b[term]==a[term]
}
pub fn new() -> (v:Machine)
    ensures v.current==election::fresh(),v.history==Map::<u64,u64>::empty(),valid_vote(v),
{ let ghost history=Map::empty(); Machine {current:election::new(),history} }
pub fn advance(v:&mut Machine,term:u64)
    requires valid_vote(*old(v)),
    ensures valid_vote(*v),v.current==election::advanced(old(v).current,term),v.history==old(v).history,
{v.current=election::advance(v.current,term);}
pub fn heartbeat(v:&mut Machine,term:u64)
    requires valid_vote(*old(v)),
    ensures valid_vote(*v),v.current==election::heartbeat_result(old(v).current,term),v.history==old(v).history,
{v.current=election::heartbeat(v.current,term);}
pub fn timeout(v:&mut Machine,node:u64)
    requires valid_vote(*old(v)),
    ensures valid_vote(*v),v.current==election::timeout_result(old(v).current,node),extends(old(v).history,v.history),
        v.history == (if old(v).current.term<u64::MAX {old(v).history.insert(v.current.term,node)} else {old(v).history}),
{
    let before=v.current.term;
    v.current=election::timeout(v.current,node);
    proof {if before<u64::MAX {v.history=v.history.insert(v.current.term,node);}}
}
pub fn request_vote(v:&mut Machine,args:Vote,last_term:u64,last_index:u64) -> (r:Reply)
    requires valid_vote(*old(v)),
    ensures valid_vote(*v),extends(old(v).history,v.history),
        v.current==election::vote_result(old(v).current,args,last_term,last_index),
        r == (Reply {term:v.current.term,granted:election::grant(old(v).current,args,last_term,last_index)}),
        v.history == (if r.granted {old(v).history.insert(args.term,args.candidate)} else {old(v).history}),
{
    let (current,r)=election::request_vote(v.current,args,last_term,last_index);
    v.current=current;
    proof {if r.granted {v.history=v.history.insert(args.term,args.candidate);}}
    r
}
pub fn reply(v:&mut Machine,term:u64,started:u64,granted:bool,votes:usize,members:usize)
    requires valid_vote(*old(v)),
    ensures valid_vote(*v),v.current==election::reply_result(old(v).current,term,started,granted,votes,members),
        v.history==old(v).history,
{v.current=election::reply(v.current,term,started,granted,votes,members);}
}
