//! DRAFT implementation progress foundation using Verus spec_fn/nat traces.
use crate::{state::Observation, io::*, persistence_spec::*, progress_spec::*};
use vstd::prelude::*;
verus! {
pub proof fn waiting_prefix(nodes: spec_fn(nat) -> Observation, events: spec_fn(nat) -> Event,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat, end: nat)
    requires start <= end, execution(nodes, events, outputs, start),
        forall |i: nat| start <= i < end ==> #[trigger] waiting_event(nodes(start), events(i)),
    ensures nodes(end) == nodes(start),
    decreases end - start,
{
    if end > start {
        waiting_prefix(nodes, events, outputs, start, (end - 1) as nat);
        let i = (end - 1) as nat;
        assert(start <= i);
        assert(i + 1 == end);
        assert(forall |j: nat| j >= start ==> #[trigger] dispatch_result(nodes(j), nodes(j + 1), events(j), outputs(j)));
        assert(dispatch_result(nodes(i), nodes(i + 1), events(i), outputs(i)));
        assert(waiting_event(nodes(start), events(i)));
        match events(i) {
            Event::Complete { operation, kind, outcome } => {
                assert(completion_error(nodes(i), operation, kind).is_some());
                assert(complete_result(nodes(i), nodes(i + 1), operation, kind, outcome, outputs(i)));
            },
            _ => {},
        }
    }
}

pub proof fn release_after_service(nodes: spec_fn(nat) -> Observation, events: spec_fn(nat) -> Event,
    outputs: spec_fn(nat) -> Result<Option<Message>, InputError>, start: nat, token: OperationId)
    requires ready(nodes(start), token), execution(nodes, events, outputs, start),
        fair_completion(nodes(start), events, start, token),
    ensures eventually_released(nodes, outputs, start),
{
    let k = choose |k: nat| k >= start && #[trigger] successful(events(k), token)
        && forall |i: nat| start <= i < k ==> #[trigger] waiting_event(nodes(start), events(i));
    waiting_prefix(nodes, events, outputs, start, k);
    assert(dispatch_result(nodes(k), nodes(k + 1), events(k), outputs(k)));
    assert(outputs(k) == Ok::<Option<Message>, InputError>(nodes(start).pending_write.unwrap().reply));
}
}
