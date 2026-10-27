//! DRAFT independent consumer; verifies the exact Cargo constructor source.
use vstd::prelude::*;
#[path = "../raft-verified/src/node_state.rs"] pub mod node_state;
#[path = "../raft-verified/src/contracts.rs"] pub mod contracts;
#[path = "observation.rs"] pub mod observation;
#[path = "../raft-spec/src/lib.rs"]
pub mod model;
#[path = "../raft-verified/src/types.rs"]
pub mod types;
#[path = "../raft-spec/src/reconcile_view.rs"]
pub mod representation;
#[path = "../raft-verified/src/state.rs"]
pub mod state;
#[path = "../raft-verified/src/io.rs"]
pub mod io;
#[path = "../raft-spec/src/node_init.rs"]
pub mod initialization;
#[path = "../raft-spec/src/persistence.rs"] pub mod persistence_spec;
#[path = "../raft-verified/src/node.rs"]
pub mod node;

verus! {
pub fn check_node_init(id: u64, members: Vec<u64>) -> (result: Result<state::Node, state::InitError>)
    ensures match result {
        Ok(n) => n.wf() && initialization::init_result(Ok(n.view()), id, members@),
        Err(e) => initialization::init_result(Err(e), id, members@),
    },
{
    state::Node::new(id, members)
}
}
