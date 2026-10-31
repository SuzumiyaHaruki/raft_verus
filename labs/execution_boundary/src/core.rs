//! The only Put implementation. Cargo erases ghost/proof code, not the handler.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer { pub ip: u32, pub port: u16 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceivedPut { pub source: Peer, pub value: u64 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendAck { pub destination: Peer, pub value: u64 }

pub open spec fn history_correct(value: u64, events: Seq<(ReceivedPut, SendAck)>) -> bool {
    &&& value == (if events.len() == 0 { 0u64 } else { events.last().0.value })
    &&& forall|i: int| 0 <= i < events.len() ==>
        (#[trigger] events[i]).1.destination == events[i].0.source
        && events[i].1.value == events[i].0.value
}

proof fn extend_history(events: Seq<(ReceivedPut, SendAck)>, previous: u64,
    request: ReceivedPut, reply: SendAck)
    requires history_correct(previous, events),
        reply.destination == request.source, reply.value == request.value,
    ensures history_correct(request.value, events.push((request, reply))),
{
    let next = events.push((request, reply));
    assert forall|i: int| 0 <= i < next.len() implies
        (#[trigger] next[i]).1.destination == next[i].0.source
        && next[i].1.value == next[i].0.value by {
        if i < events.len() { assert(next[i] == events[i]); }
        else { assert(i == events.len()); }
    }
}

// Private fields prevent the unverified adapter from writing state or history.
pub struct Core {
    value: u64,
    events: Ghost<Seq<(ReceivedPut, SendAck)>>,
}

impl Core {
    #[verifier::type_invariant]
    closed spec fn well_formed(self) -> bool {
        history_correct(self.value, self.events@)
    }

    pub closed spec fn stored(&self) -> u64 { self.value }
    pub closed spec fn history(&self) -> Seq<(ReceivedPut, SendAck)> { self.events@ }

    pub fn new() -> (result: Self)
        ensures result.stored() == 0, result.history() == Seq::<(ReceivedPut, SendAck)>::empty(),
    {
        Self { value: 0, events: Ghost(Seq::empty()) }
    }

    // No caller-supplied protocol invariant or precondition is required.
    pub fn handle(&mut self, request: ReceivedPut) -> (reply: SendAck)
        ensures self.stored() == request.value,
            reply.destination == request.source, reply.value == request.value,
            self.history() == old(self).history().push((request, reply)),
    {
        proof { use_type_invariant(&*self); }
        let ghost previous = self.value;
        let ghost before = self.events@;
        let reply = SendAck { destination: request.source, value: request.value };
        proof { extend_history(before, previous, request, reply); }
        *self = Core { value: request.value, events: Ghost(before.push((request, reply))) };
        reply
    }

    pub fn value(&self) -> (result: u64)
        ensures result == self.stored(),
    { self.value }

    pub fn history_is_correct(&self)
        ensures history_correct(self.stored(), self.history()),
    { proof { use_type_invariant(self); } }
}
}
