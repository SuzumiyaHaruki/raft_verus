#[path = "src/core.rs"]
mod core;
use core::{Core, ReceivedPut, SendAck, history_correct};
use vstd::prelude::*;

verus! {
// No assumption about correct replies/steps: the actual constructor and handler
// establish/preserve a private type invariant and append the actual reply.
pub fn finite_processing_history(server: &Core)
    ensures history_correct(server.stored(), server.history()),
        server.history().len() == 0 ==> server.stored() == 0,
        server.history().len() > 0 ==> server.stored() == server.history().last().0.value,
        forall|i: int| 0 <= i < server.history().len() ==>
            (#[trigger] server.history()[i]).1.value == server.history()[i].0.value
            && server.history()[i].1.destination == server.history()[i].0.source,
{
    server.history_is_correct();
}

// An arbitrary finite sequence of real calls, including the empty sequence.
// This executable proof caller uses the same functions as the async service.
pub fn any_finite_calls(requests: &Vec<ReceivedPut>) -> (result: (u64, Vec<SendAck>))
    ensures result.1.len() == requests.len(),
        result.0 == (if requests.len() == 0 { 0u64 } else { requests@.last().value }),
        forall|i: int| 0 <= i < requests.len() ==>
            (#[trigger] result.1@[i]).value == requests@[i].value
            && result.1@[i].destination == requests@[i].source,
{
    let mut server = Core::new();
    let mut replies = Vec::<SendAck>::new();
    let mut i: usize = 0;
    while i < requests.len()
        invariant i <= requests.len(), replies.len() == i, server.history().len() == i,
            forall|j: int| 0 <= j < i ==>
                (#[trigger] server.history()[j]).0 == requests@[j]
                && server.history()[j].1 == replies@[j],
        decreases requests.len() - i,
    {
        let reply = server.handle(requests[i]);
        replies.push(reply);
        i = i + 1;
    }
    finite_processing_history(&server);
    proof {
        assert forall|j: int| 0 <= j < requests.len() implies
            (#[trigger] replies@[j]).value == requests@[j].value
            && replies@[j].destination == requests@[j].source by {
            assert(server.history()[j].0 == requests@[j]);
            assert(server.history()[j].1 == replies@[j]);
        }
    }
    (server.value(), replies)
}
}
