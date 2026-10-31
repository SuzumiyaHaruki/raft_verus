//! Independent callers of the actual constructors, begin and complete.
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

pub fn fresh_finite_calls(inputs: &Vec<Input>) -> (s: Core)
    ensures s.trace().origin == Origin::Fresh,
        s.trace().calls == inputs.len(), history_ok(s.counter(), s.waiting(), s.stored(), s.trace()),
{
    let mut s = Core::new();
    finite_calls(&mut s, inputs);
    s
}

pub fn recovered_finite_calls(record: Record, inputs: &Vec<Input>) -> (result: Option<Core>)
    ensures result.is_some() <==> record.id < u64::MAX,
        result.is_some() ==> {
            let s = result.unwrap();
            &&& s.trace().origin == Origin::Recovered(record)
            &&& s.trace().calls == inputs.len()
            &&& s.counter() > record.id
            &&& history_ok(s.counter(), s.waiting(), s.stored(), s.trace())
        },
{
    match Core::recover(record) {
        None => None,
        Some(mut s) => { finite_calls(&mut s, inputs); Some(s) },
    }
}

// Stronger than empty Pending: the LAST accepted operation has an Ack.
// Client receipt and faithful file retention/readback are HOST premises.
pub open spec fn restart_point(s: &Core, record: Record) -> bool {
    &&& s.waiting().is_none()
    &&& s.trace().accepted.len() > 0
    &&& 0 <= s.trace().last_ack < s.trace().finished.len()
    &&& s.trace().finished[s.trace().last_ack].reply.is_some()
    &&& s.trace().finished[s.trace().last_ack].reply.unwrap().id == s.trace().accepted.last().write.id
    &&& record.id == s.trace().accepted.last().write.id
    &&& record.value == s.trace().accepted.last().write.request.value
}

// `record` denotes the bytes decoded after restart. The precondition explicitly
// binds them to the last successful accepted operation, not to a desired state.
pub fn recover_retained_record(before: &Core, record: Record) -> (after: Core)
    requires restart_point(before, record),
    ensures after.stored() == before.stored(), after.counter() == before.counter(),
        after.waiting().is_none(), after.trace().origin == Origin::Recovered(record),
        after.trace().calls == 0, after.trace().accepted.len() == 0, after.trace().finished.len() == 0,
        forall|i: int| 0 <= i < before.trace().accepted.len()
            ==> (#[trigger] before.trace().accepted[i]).write.id < after.counter(),
{
    before.history_is_correct();
    let ghost h = before.trace();
    proof {
        assert(h.accepted[h.accepted.len() - 1].write.id == record.id);
        assert(record.id + 1 == before.counter());
        assert(h.finished[h.last_ack].reply.unwrap().id == record.id);
        assert(record.id - base_next(h.origin) == h.accepted.len() - 1);
    }
    let after = Core::recover(record).unwrap();
    after
}

pub fn old_notice_is_rejected(s: &mut Core, notice: Completion) -> (reply: Option<Ack>)
    requires notice.id < base_next(old(s).trace().origin),
    ensures reply.is_none(), s.stored() == old(s).stored(), s.waiting() == old(s).waiting(),
{
    s.history_is_correct();
    s.complete(notice)
}

pub fn failure_is_not_a_restart_point(a: Put, b: Put) -> (s: Core)
    ensures s.waiting().is_none(), s.counter() == 2,
        !restart_point(&s, Record { id: 0, value: a.value }),
{
    let mut s = Core::new();
    s.begin(a);
    s.complete(Completion { id: 0, success: true });
    s.begin(b);
    s.complete(Completion { id: 1, success: false });
    s
}

// A concrete composition using new/begin/complete/recover and an arbitrary
// post-restart suffix. The local Record stands for faithful external write /
// retention / readback; this caller does NOT execute or verify a filesystem.
pub fn one_confirmed_restart(requests: &Vec<Put>, suffix: &Vec<Input>) -> (after: Core)
    requires 0 < requests.len(), requests.len() <= u64::MAX,
    ensures after.trace().calls == suffix.len(),
        after.counter() >= requests.len(),
        after.trace().origin == Origin::Recovered(Record {
            id: (requests.len() - 1) as u64, value: requests@.last().value }),
        history_ok(after.counter(), after.waiting(), after.stored(), after.trace()),
{
    let mut before = Core::new();
    let mut record = Record { id: 0, value: 0 };
    let mut i: usize = 0;
    while i < requests.len()
        invariant i <= requests.len(), before.counter() == i, before.waiting().is_none(),
            i > 0 ==> restart_point(&before, record),
            i > 0 ==> record.id == i - 1 && record.value == requests@[i - 1].value,
        decreases requests.len() - i,
    {
        let Begin::Write(w) = before.begin(requests[i]) else { unreached() };
        let reply = before.complete(Completion { id: w.id, success: true });
        assert(reply == Some(Ack { id: w.id, destination: w.request.source, value: w.request.value }));
        record = Record { id: w.id, value: w.request.value };
        before.history_is_correct();
        i = i + 1;
    }
    let mut after = recover_retained_record(&before, record);
    finite_calls(&mut after, suffix);
    after
}
}
