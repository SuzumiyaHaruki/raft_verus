//! DRAFT implementation progress foundation using Verus spec_fn/nat traces.
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
    if end > start {
        waiting_prefix(nodes, events, outputs, start, (end - 1) as nat);
        assert(dispatch_result(nodes((end - 1) as nat), nodes(end), events((end - 1) as nat), outputs((end - 1) as nat)));
    }
}

pub proof fn release_after_service(nodes: spec_fn(nat) -> Node, events: spec_fn(nat) -> Event,
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
