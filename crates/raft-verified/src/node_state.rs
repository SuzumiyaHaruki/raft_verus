//! DRAFT editable representation scaffold. Observation is the stable inspection vocabulary.
use crate::state::Observation;
use vstd::prelude::*;

verus! {
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub state: Observation,
}

impl Node {
    pub open spec fn view(self) -> Observation { self.state }
    pub open spec fn wf(self) -> bool { true }

    /// Inspection adapter, not a protocol transition. Its correspondence is checked independently.
    pub fn observe(&self) -> (result: &Observation)
        requires self.wf(),
        ensures *result == self.view(),
    { &self.state }

    /// Boundary-test adapter only; this is not an authorized recovery transition.
    pub fn from_observation(state: Observation) -> (result: Self)
        ensures result.wf(), result.view() == state,
    { Node { state } }
}
}
