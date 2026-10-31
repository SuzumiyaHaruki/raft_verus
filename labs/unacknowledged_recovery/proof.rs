//! Independent callers of the actual core. Finite-call helpers are copied from
//! quiescent_recovery; the pending-write composition below does NOT require an old Ack.
#[path = "src/core.rs"] mod core;
use core::*;
use vstd::prelude::*;

verus! {
pub fn finite_history(s: &Core)
    ensures history_ok(s.counter(), s.waiting(), s.stored(), s.trace()),
        forall|i: int| 0 <= i < s.trace().finished.len() && (#[trigger] s.trace().finished[i]).reply.is_some() ==> {
            let f = s.trace().finished[i];
            let a = f.reply.unwrap();
            let j = a.id - base_next(s.trace().origin);
            &&& f.notice == (Completion { id: a.id, success: true })
            &&& 0 <= j < s.trace().accepted.len()
            &&& s.trace().accepted[j].at < f.at
            &&& a.destination == s.trace().accepted[j].write.request.source
            &&& a.value == s.trace().accepted[j].write.request.value
        },
{
    s.history_is_correct();
}

#[derive(Clone, Copy)]
pub enum Input { Put(Put), Complete(Completion) }

pub fn finite_calls(s: &mut Core, inputs: &Vec<Input>)
    ensures s.trace().origin == old(s).trace().origin,
        s.trace().calls == old(s).trace().calls + inputs.len(),
        s.counter() >= old(s).counter(),
        history_ok(s.counter(), s.waiting(), s.stored(), s.trace()),
{
    let ghost initial = s.trace();
    let ghost start_next = s.counter();
    let mut i: usize = 0;
    while i < inputs.len()
        invariant i <= inputs.len(), s.trace().origin == initial.origin,
            s.trace().calls == initial.calls + i, s.counter() >= start_next,
        decreases inputs.len() - i,
    {
        match inputs[i] {
            Input::Put(p) => { s.begin(p); },
            Input::Complete(c) => { s.complete(c); },
        }
        i = i + 1;
    }
    finite_history(s);
}

// This is a relation between a pending Write and decoded bytes, NOT evidence
// of I/O. The host must actually finish that write, retain it, and read it back.
pub open spec fn pending_record(s: &Core, record: Record) -> bool {
    &&& s.waiting().is_some()
    &&& record.id == s.waiting().unwrap().id
    &&& record.value == s.waiting().unwrap().request.value
}

pub fn recover_written_pending(before: &Core, record: Record) -> (after: Core)
    requires pending_record(before, record),
    ensures after.stored() == record.value,
        after.counter() == record.id + 1, after.counter() == before.counter(),
        after.waiting().is_none(), after.trace().origin == Origin::Recovered(record),
        after.trace().calls == 0, after.trace().accepted.len() == 0, after.trace().finished.len() == 0,
        forall|i: int| 0 <= i < before.trace().accepted.len()
            ==> (#[trigger] before.trace().accepted[i]).write.id < after.counter(),
        forall|i: int| 0 <= i < before.trace().finished.len() && (#[trigger] before.trace().finished[i]).reply.is_some()
            ==> before.trace().finished[i].reply.unwrap().id < record.id,
{
    before.history_is_correct();
    // Pending is the last allocated operation. Its ID has NOT yet authorized
    // an Ack, even when the host has already written its bytes successfully.
    assert(record.id + 1 == before.counter());
    Core::recover(record).unwrap()
}

// `record` is supplied by faithful external write/retention/readback. Its
// relation to the actual Write is checked below; no complete is invented here.
pub fn begin_then_recover(before: &mut Core, request: Put, record: Record) -> (after: Core)
    requires old(before).waiting().is_none(), old(before).counter() < u64::MAX,
        record.id == old(before).counter(), record.value == request.value,
    ensures before.stored() == old(before).stored(),
        before.waiting() == Some(Write { id: record.id, request }),
        before.trace().finished == old(before).trace().finished,
        after.stored() == request.value, after.counter() == record.id + 1,
        after.counter() == before.counter(), after.waiting().is_none(),
        after.trace().origin == Origin::Recovered(record),
        after.trace().calls == 0, after.trace().accepted.len() == 0, after.trace().finished.len() == 0,
        forall|i: int| 0 <= i < before.trace().accepted.len()
            ==> (#[trigger] before.trace().accepted[i]).write.id < after.counter(),
        forall|i: int| 0 <= i < before.trace().finished.len() && (#[trigger] before.trace().finished[i]).reply.is_some()
            ==> before.trace().finished[i].reply.unwrap().id < record.id,
{
    let Begin::Write(w) = before.begin(request) else { unreached() };
    assert(w == Write { id: record.id, request });
    assert(record == Record { id: w.id, value: w.request.value });
    recover_written_pending(before, record)
}

pub fn written_restart_with_suffix(before: &mut Core, request: Put, record: Record,
    suffix: &Vec<Input>) -> (after: Core)
    requires old(before).waiting().is_none(), old(before).counter() < u64::MAX,
        record.id == old(before).counter(), record.value == request.value,
    ensures before.stored() == old(before).stored(),
        before.waiting() == Some(Write { id: record.id, request }),
        before.trace().finished == old(before).trace().finished,
        after.trace().origin == Origin::Recovered(record), after.trace().calls == suffix.len(),
        after.counter() >= before.counter(),
        forall|i: int| 0 <= i < before.trace().accepted.len()
            ==> (#[trigger] before.trace().accepted[i]).write.id < after.counter(),
        history_ok(after.counter(), after.waiting(), after.stored(), after.trace()),
{
    let mut after = begin_then_recover(before, request, record);
    finite_calls(&mut after, suffix);
    after
}

// Nonempty, machine-checked witness: only op0 completes. Op1 remains Pending,
// with no extra finished entry or Ack, while the recovered base becomes 9.
pub fn witness_seven_then_nine(a: Peer, b: Peer) -> (pair: (Core, Core))
    ensures pair.0.stored() == 7, pair.0.counter() == 2,
        pair.0.waiting() == Some(Write { id: 1, request: Put { source: b, value: 9 } }),
        pair.0.trace().finished.len() == 1,
        pair.0.trace().finished[0].reply == Some(Ack { id: 0, destination: a, value: 7 }),
        pair.1.stored() == 9, pair.1.counter() == 2, pair.1.waiting().is_none(),
        pair.1.trace().origin == Origin::Recovered(Record { id: 1, value: 9 }),
        pair.1.trace().calls == 0, pair.1.trace().accepted.len() == 0, pair.1.trace().finished.len() == 0,
{
    let mut before = Core::new();
    let Begin::Write(w) = before.begin(Put { source: a, value: 7 }) else { unreached() };
    let reply = before.complete(Completion { id: w.id, success: true });
    assert(reply == Some(Ack { id: 0, destination: a, value: 7 }));
    let after = begin_then_recover(&mut before, Put { source: b, value: 9 }, Record { id: 1, value: 9 });
    (before, after)
}

pub fn old_notice_is_rejected(s: &mut Core, notice: Completion) -> (reply: Option<Ack>)
    requires notice.id < base_next(old(s).trace().origin),
    ensures reply.is_none(), s.stored() == old(s).stored(), s.waiting() == old(s).waiting(),
{
    s.history_is_correct();
    s.complete(notice)
}
}

