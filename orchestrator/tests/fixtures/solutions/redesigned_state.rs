//! SIMULATED alternative design: nested storage plus a representation invariant.
use crate::state::Observation;
use vstd::prelude::*;
verus! {
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Core { pub image: Observation }
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node { pub core: Core, pub issued: u64 }
impl Node {
    pub open spec fn view(self) -> Observation { self.core.image }
    pub open spec fn wf(self) -> bool { self.issued == self.core.image.serial }
    pub fn observe(&self) -> (result: &Observation)
        requires self.wf(),
        ensures *result == self.view(),
    { &self.core.image }
    pub fn from_observation(state: Observation) -> (result: Self)
        ensures result.wf(), result.view() == state,
    {
        let issued = state.serial;
        Node { core: Core { image: state }, issued }
    }
}
}
