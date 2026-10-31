//! Actual executable state machine. Success means a received notification,
//! not a machine-checked claim that an external I/O operation happened.
use vstd::prelude::*;

verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer { pub ip: u32, pub port: u16 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Put { pub source: Peer, pub value: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Write { pub id: u64, pub request: Put }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Begin { Write(Write), Busy, Exhausted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Completion { pub id: u64, pub success: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ack { pub id: u64, pub destination: Peer, pub value: u64 }

// Ghost histories are appended by the actual entrypoints, never supplied by
// callers. `at` is a ghost invocation index, not a client request identifier.
pub struct Accepted { pub write: Write, pub at: int }
pub struct Finished { pub notice: Completion, pub reply: Option<Ack>, pub at: int }
pub struct History {
    pub accepted: Seq<Accepted>,
    pub finished: Seq<Finished>,
    pub calls: int,
    pub last_ack: int,
}

pub open spec fn begin_result(next: u64, pending: Option<Write>, request: Put) -> Begin {
    if pending.is_some() { Begin::Busy }
    else if next == u64::MAX { Begin::Exhausted }
    else { Begin::Write(Write { id: next, request }) }
}
pub open spec fn matches(pending: Option<Write>, notice: Completion) -> bool {
    pending.is_some() && pending.unwrap().id == notice.id
}
pub open spec fn completion_result(pending: Option<Write>, notice: Completion) -> Option<Ack> {
    if matches(pending, notice) && notice.success {
        Some(Ack { id: pending.unwrap().id, destination: pending.unwrap().request.source,
            value: pending.unwrap().request.value })
    } else { None }
}
pub open spec fn begin_history(h: History, result: Begin) -> History {
    History { calls: h.calls + 1, last_ack: h.last_ack, finished: h.finished,
        accepted: match result {
            Begin::Write(w) => h.accepted.push(Accepted { write: w, at: h.calls }),
            _ => h.accepted,
        } }
}
pub open spec fn finish_history(h: History, notice: Completion, reply: Option<Ack>) -> History {
    History { calls: h.calls + 1, accepted: h.accepted,
        last_ack: if reply.is_some() { h.finished.len() as int } else { h.last_ack },
        finished: h.finished.push(Finished { notice, reply, at: h.calls }) }
}

pub open spec fn history_ok(next: u64, pending: Option<Write>, value: u64, h: History) -> bool {
    &&& h.calls >= 0
    &&& h.accepted.len() == next
    &&& forall|i: int| 0 <= i < h.accepted.len() ==>
        (#[trigger] h.accepted[i]).write.id == i && 0 <= h.accepted[i].at < h.calls
    &&& forall|i: int| 0 <= i < h.finished.len() ==>
        0 <= (#[trigger] h.finished[i]).at < h.calls
        && (h.finished[i].reply.is_some() ==> {
            let a = h.finished[i].reply.unwrap();
            &&& a.id < next
            &&& h.finished[i].notice == (Completion { id: a.id, success: true })
            &&& h.accepted[a.id as int].at < h.finished[i].at
            &&& a.destination == h.accepted[a.id as int].write.request.source
            &&& a.value == h.accepted[a.id as int].write.request.value
        })
    &&& forall|i: int, j: int| 0 <= i < j < h.finished.len() ==>
        (#[trigger] h.finished[i]).at < (#[trigger] h.finished[j]).at
        && (h.finished[i].reply.is_some() && h.finished[j].reply.is_some() ==>
            h.finished[i].reply.unwrap().id < h.finished[j].reply.unwrap().id)
    &&& (pending.is_some() ==> {
        let p = pending.unwrap();
        &&& p.id + 1 == next
        &&& h.accepted[p.id as int].write == p
        &&& forall|i: int| 0 <= i < h.finished.len() && (#[trigger] h.finished[i]).reply.is_some()
            ==> h.finished[i].reply.unwrap().id < p.id
    })
    &&& -1 <= h.last_ack < h.finished.len()
    &&& (h.last_ack == -1 ==> value == 0)
    &&& (h.last_ack >= 0 ==> h.finished[h.last_ack].reply.is_some()
        && value == h.finished[h.last_ack].reply.unwrap().value)
    &&& forall|i: int| h.last_ack < i < h.finished.len() ==> (#[trigger] h.finished[i]).reply.is_none()
}

pub struct Core {
    completed: u64,
    next: u64,
    pending: Option<Write>,
    history: Ghost<History>,
}

impl Core {
    #[verifier::type_invariant]
    closed spec fn well_formed(self) -> bool {
        history_ok(self.next, self.pending, self.completed, self.history@)
    }
    pub closed spec fn stored(&self) -> u64 { self.completed }
    pub closed spec fn waiting(&self) -> Option<Write> { self.pending }
    pub closed spec fn counter(&self) -> u64 { self.next }
    pub closed spec fn trace(&self) -> History { self.history@ }

    pub fn new() -> (s: Self)
        ensures s.stored() == 0, s.waiting().is_none(), s.counter() == 0,
            s.trace().calls == 0, s.trace().accepted.len() == 0, s.trace().finished.len() == 0,
    {
        Self { completed: 0, next: 0, pending: None,
            history: Ghost(History { accepted: Seq::empty(), finished: Seq::empty(),
                calls: 0, last_ack: -1 }) }
    }

    // MAX is the exhaustion sentinel; allocated IDs are 0..MAX (exclusive).
    // Busy is checked first and never replaces an existing Pending.
    pub fn begin(&mut self, request: Put) -> (result: Begin)
        ensures result == begin_result(old(self).counter(), old(self).waiting(), request),
            self.stored() == old(self).stored(),
            self.counter() == match result { Begin::Write(_) => old(self).counter() + 1, _ => old(self).counter() as int },
            self.waiting() == match result { Begin::Write(w) => Some(w), _ => old(self).waiting() },
            self.trace() == begin_history(old(self).trace(), result),
    {
        proof { use_type_invariant(&*self); }
        let result;
        let mut next = self.next;
        let mut pending = self.pending;
        if pending.is_some() { result = Begin::Busy; }
        else if next == u64::MAX { result = Begin::Exhausted; }
        else {
            let write = Write { id: next, request };
            next = next + 1;
            pending = Some(write);
            result = Begin::Write(write);
        }
        let ghost h = begin_history(self.history@, result);
        *self = Core { completed: self.completed, next, pending, history: Ghost(h) };
        result
    }

    // Arbitrary completion IDs and success/failure notifications are legal inputs.
    pub fn complete(&mut self, notice: Completion) -> (reply: Option<Ack>)
        ensures reply == completion_result(old(self).waiting(), notice),
            self.counter() == old(self).counter(),
            self.waiting() == if matches(old(self).waiting(), notice) { None } else { old(self).waiting() },
            self.stored() == if reply.is_some() { reply.unwrap().value } else { old(self).stored() },
            self.trace() == finish_history(old(self).trace(), notice, reply),
    {
        proof { use_type_invariant(&*self); }
        let mut reply = None;
        let mut pending = self.pending;
        let mut completed = self.completed;
        if let Some(p) = pending {
            if notice.id == p.id {
                pending = None;
                if notice.success {
                    completed = p.request.value;
                    reply = Some(Ack { id: p.id, destination: p.request.source, value: p.request.value });
                }
            }
        }
        let ghost h = finish_history(self.history@, notice, reply);
        *self = Core { completed, next: self.next, pending, history: Ghost(h) };
        reply
    }

    pub fn value(&self) -> (value: u64) ensures value == self.stored() { self.completed }
    pub fn pending(&self) -> (pending: Option<Write>) ensures pending == self.waiting() { self.pending }
    pub fn history_is_correct(&self)
        ensures history_ok(self.counter(), self.waiting(), self.stored(), self.trace()),
    { proof { use_type_invariant(self); } }
}
}

#[cfg(test)]
mod boundary {
    use super::*;
    #[test]
    fn identifier_boundary_does_not_wrap() {
        // Arithmetic-only fixture after ghost erasure, not a claimed history
        // of MAX-1 executed requests. Verus separately checks reachable states.
        let mut core = Core::new();
        core.next = u64::MAX - 1;
        let request = Put { source: Peer { ip: 1, port: 2 }, value: 3 };
        let Begin::Write(w) = core.begin(request) else { panic!("last allocatable ID") };
        assert_eq!(w.id, u64::MAX - 1);
        assert_eq!(core.begin(request), Begin::Busy);
        assert!(core.complete(Completion { id: w.id, success: true }).is_some());
        assert_eq!(core.begin(request), Begin::Exhausted);
        assert_eq!(core.begin(request), Begin::Exhausted);
        assert_eq!(core.next, u64::MAX);
    }
}
