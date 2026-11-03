//! Maintenance-only defects; never copied into experiment input.
#[path="api.rs"] mod vote_api;
use vote_api::*;
use vstd::prelude::*;
verus! {
#[cfg(any(bad_vote,bad_term))]
fn bad_recover(c:Config,r:Record) -> (out:vote_api::View)
    requires valid_record(c,r),
    ensures out==recovered_view(c,r),
{
    let mut state=r.state;
    #[cfg(bad_vote)] {state.voted_for=None;}
    #[cfg(bad_term)] {state.term=0;}
    vote_api::View{config:c,state,role:Role::Follower,next:r.op+1,pending:None,halted:false,round:None,self_saved:false}
}
// Mechanical observation retention; no uniqueness or majority reasoning here.
#[cfg(bad_history)]
fn retained(old:&Vec<Choice>) -> (out:Vec<Choice>) ensures out@==old@,
{ Vec::new() }
#[cfg(control)]
fn retained(old:&Vec<Choice>) -> (out:Vec<Choice>) ensures out@==old@,
{ old.clone() }
// The actual writer owns the retained slot. An arbitrary well-formed record is
// still not its latest successful record. This is a source rule, not Raft logic.
#[cfg(any(bad_source,control))]
fn source(latest:Record,unrelated:Record) -> (read:Record) ensures read==latest,
{
    #[cfg(bad_source)] {unrelated}
    #[cfg(control)] {latest}
}
}
#[test]
fn fixture() {
    let record=Record{op:3,state:Persistent{term:7,voted_for:Some(2),log:Log{term:4,index:5}}};
    let config=Config{node:1,members:3};
    #[cfg(any(bad_vote,bad_term))] {
        let v=bad_recover(config,record);
        assert_eq!(v.state,record.state,"recovery must preserve actual retained term/vote");
    }
    #[cfg(any(bad_history,control))] {
        let h=vec![Choice{node:1,term:7,candidate:2}];
        assert_eq!(retained(&h),h,"restart erased historical observation");
    }
    #[cfg(any(bad_source,control))] {
        let stale=Record{op:2,state:Persistent{term:6,..record.state}};
        assert_eq!(source(record,stale),record,"recovery used stale or arbitrary record");
    }
}
