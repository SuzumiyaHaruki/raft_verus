//! DRAFT independent consumer; verifies the exact Cargo constructor source.
use vstd::prelude::*;
#[path = "../raft-spec/src/lib.rs"]
pub mod model;
#[path = "../raft-verified/src/reconcile.rs"]
pub mod reconcile;
#[path = "../raft-spec/src/reconcile_view.rs"]
pub mod representation;
#[path = "../raft-verified/src/state.rs"]
pub mod state;
#[path = "../raft-verified/src/io.rs"]
pub mod io;
#[path = "../raft-spec/src/node_init.rs"]
pub mod initialization;
#[path = "../raft-verified/src/node.rs"]
pub mod node;

verus! {
pub fn check_node_init(id: u64, members: Vec<u64>) -> (result: Result<state::Node, state::InitError>)
    ensures initialization::init_result(result, id, members@),
{
    state::Node::new(id, members)
}
}
