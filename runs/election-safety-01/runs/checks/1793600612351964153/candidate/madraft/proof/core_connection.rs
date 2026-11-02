#[path="../../tasks/election_safety/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
use vstd::prelude::*;
use vote_api::*;
verus! {
pub fn initialized(c:Config,initial:Initial) {
    let core=vote_core::Core::new(c,initial);
    if let Some(core)=core { let actual=core.inspect(); assert(actual==fresh_view(c,initial)); }
}
}
