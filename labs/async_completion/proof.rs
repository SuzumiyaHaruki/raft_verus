#[path = "src/core.rs"]
mod core;
use core::*;
use vstd::prelude::*;

verus! {
pub fn finite_history(s: &Core)
    ensures history_ok(s.counter(), s.waiting(), s.stored(), s.trace()),
        forall|i: int| 0 <= i < s.trace().finished.len() && (#[trigger] s.trace().finished[i]).reply.is_some() ==> {
            let f = s.trace().finished[i];
            let a = f.reply.unwrap();
            &&& f.notice == (Completion { id: a.id, success: true })
            &&& a.id < s.trace().accepted.len()
            &&& s.trace().accepted[a.id as int].at < f.at
            &&& a.destination == s.trace().accepted[a.id as int].write.request.source
            &&& a.value == s.trace().accepted[a.id as int].write.request.value
        },
        forall|i: int, j: int| 0 <= i < j < s.trace().finished.len()
            && (#[trigger] s.trace().finished[i]).reply.is_some()
            && (#[trigger] s.trace().finished[j]).reply.is_some()
            ==> s.trace().finished[i].reply.unwrap().id != s.trace().finished[j].reply.unwrap().id,
{
    s.history_is_correct();
}

#[derive(Clone, Copy)]
pub enum Input { Put(Put), Complete(Completion) }

// No restriction on IDs, outcomes, duplicates, interleavings or u64 payloads.
// The same real constructor/begin/complete are used by the async adapter.
pub fn any_finite_calls(inputs: &Vec<Input>) -> (s: Core)
    ensures s.trace().calls == inputs.len(),
        history_ok(s.counter(), s.waiting(), s.stored(), s.trace()),
{
    let mut s = Core::new();
    let mut i: usize = 0;
    while i < inputs.len()
        invariant i <= inputs.len(), s.trace().calls == i,
        decreases inputs.len() - i,
    {
        match inputs[i] {
            Input::Put(request) => { s.begin(request); },
            Input::Complete(notice) => { s.complete(notice); },
        }
        i = i + 1;
    }
    finite_history(&s);
    s
}
}
