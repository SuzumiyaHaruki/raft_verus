//! Full composition is under development; exported APIs will not assume invariants.
#[path = "execution.rs"]
mod model;
pub use model::*;

use vstd::prelude::*;
use crate::targets;
verus! {
// Unfinished composition goals. These bodies intentionally fail their stated
// postconditions; no reachable-state invariant is introduced as a premise.
// Local representation induction does not establish authentic durable votes.
pub proof fn prove_election_safety(run:Execution)
    requires execution(run),
    ensures targets::election_safety(observe(run)),
{ observation_shape(run); state_induction(run,0); }

// Requires provenance of AppendEntries and unique command creation per term
// and index, across histories, including justified compacted prefixes.
pub proof fn prove_log_matching(run:Execution)
    requires execution(run),
    ensures targets::log_matching(observe(run)),
{ observation_shape(run); state_induction(run,0); }

// Requires durable quorum acknowledgments and election up-to-date induction.
pub proof fn prove_leader_completeness(run:Execution)
    requires execution(run),
    ensures targets::leader_completeness(observe(run)),
{ observation_shape(run); state_induction(run,0); }

// Requires cross-history committed-prefix agreement, including snapshot decode.
pub proof fn prove_state_machine_safety(run:Execution)
    requires execution(run),
    ensures targets::state_machine_safety(observe(run)),
{ observation_shape(run); state_induction(run,0); }
}
