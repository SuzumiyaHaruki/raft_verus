//! DRAFT independent bridge between executable inspection and the generated representation.
use crate::state::{Node, Observation};
use vstd::prelude::*;

verus! {
pub fn check_observe(n: &Node) -> (result: &Observation)
    requires n.wf(),
    ensures *result == n.view(),
{ n.observe() }

pub fn check_observation_input(state: Observation) -> (result: Node)
    ensures result.wf(), result.view() == state,
{ Node::from_observation(state) }
}
