//! DRAFT proof targets with empty bodies; no induction strategy is supplied.
use crate::{state::Node, io::*, persistence_spec::*, progress_spec::*};
use vstd::prelude::*;
verus! {
pub proof fn waiting_prefix(nodes: spec_fn(nat) -> Node, events: spec_fn(nat) -> Event,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat, end: nat)
    requires start <= end, execution(nodes, events, outputs, start),
        forall |i: nat| start <= i < end ==> #[trigger] waiting_event(nodes(start), events(i)),
    ensures nodes(end) == nodes(start),
    decreases end - start,
{
    // Proof strategy intentionally absent; the unchanged target must be established.
}

pub proof fn release_after_service(nodes: spec_fn(nat) -> Node, events: spec_fn(nat) -> Event,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat, token: OperationId)
    requires ready(nodes(start), token), execution(nodes, events, outputs, start),
        fair_completion(nodes(start), events, start, token),
    ensures eventually_released(nodes, outputs, start),
{
    // Proof strategy intentionally absent; the unchanged target must be established.
}
}
