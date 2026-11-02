//! Maintenance-only mathematical/observation defects. Never experiment inputs.
#[path="api.rs"] mod vote_api;
#[path="../../requirements/targets.rs"] mod targets;
#[path="observation.rs"] mod election_observation;
use vote_api::*;
use election_observation::{Change,Election};
use vstd::prelude::*;
verus! {
#[cfg(bad_half)]
fn half(members:u64,count:u64) -> (won:bool)
    ensures won ==> count>members/2,
{ count>=members/2 }
#[cfg(bad_unsaved)]
fn unsaved(saved:bool) -> (won:bool) ensures won ==> saved,
{ true }
#[cfg(bad_omission)]
fn omitted(change:Change) -> (event:Option<Election>)
    ensures event.is_some()==election_observation::elects(change),
{ None }
#[cfg(bad_erasure)]
fn erased(events:&Vec<Election>) -> (retained:Vec<Election>)
    ensures retained@==events@,
{ Vec::new() }
#[cfg(control)]
pub proof fn specification_controls()
    ensures !majority(Config{node:0,members:4},Set::<u64>::empty().insert(0).insert(1)),
        majority(Config{node:0,members:3},Set::<u64>::empty().insert(0).insert(1)),
        !promotion_allowed(vote_api::View{config:Config{node:0,members:1},state:Persistent{term:1,voted_for:Some(0),log:Log{term:0,index:0}},
            role:Role::Candidate,next:1,pending:None,halted:false,round:Some(1),self_saved:false},Set::<u64>::empty().insert(0)),
{
    broadcast use {vstd::set::group_set_axioms,vstd::set_lib::group_set_properties};
    let s=Set::<u64>::empty().insert(0).insert(1);
    assert forall|v:u64| #[trigger] s.contains(v) implies v<3 by {}
}
}
#[test]
fn fixture() {
    let v=vote_api::View{config:Config{node:0,members:3},state:Persistent{term:1,voted_for:Some(0),log:Log{term:0,index:0}},
        role:Role::Candidate,next:1,pending:None,halted:false,round:Some(1),self_saved:true};
    let change=Change{node:0,before:v,after:vote_api::View{role:Role::Leader,..v}};
    #[cfg(bad_half)] assert!(!half(4,2),"half is not a strict majority");
    #[cfg(bad_unsaved)] assert!(!unsaved(false),"unsaved self vote cannot authorize promotion");
    #[cfg(bad_omission)] assert_eq!(omitted(change),Some(Election{node:0,term:1}),"actual election was omitted");
    #[cfg(bad_erasure)] assert_eq!(erased(&vec![Election{node:0,term:1}]),vec![Election{node:0,term:1}],"demotion erased election history");
    #[cfg(control)] {
        assert_eq!(election_observation::observe(change),Some(Election{node:0,term:1}));
        let down=Change{node:0,before:change.after,after:vote_api::View{role:Role::Follower,state:Persistent{term:2,..v.state},..v}};
        let history:Vec<_>=[change,down].into_iter().filter_map(election_observation::observe).collect();
        assert_eq!(history,vec![Election{node:0,term:1}]);
    }
}
