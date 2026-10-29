//! Executable toy only: no Raft algorithm, invariant or solution is supplied.
#[path = "support.rs"]
mod support;
use support::*;
use vstd::prelude::*;
verus! {
pub open spec fn message(recipient: nat, body: u64) -> Envelope<u64> {
    Envelope { group: 1, sender: 0, recipient, body }
}
pub open spec fn emitted(o: (u64, u64)) -> Seq<Envelope<u64>> {
    seq![message(1, o.0), message(2, o.1)]
}
pub open spec fn receive(i: Option<Delivery<u64>>) -> Option<Delivery<u64>> { i }
pub open spec fn payload(i: Option<Delivery<u64>>) -> Option<u64> {
    match i { Some(d) => Some(d.message.body), None => None }
}
pub open spec fn initial(s: u64) -> bool { s == 0 }
pub open spec fn step(s: u64, i: Option<Delivery<u64>>, t: u64, o: (u64, u64)) -> bool {
    t == (if s < 100 { s + 1 } else { s as int }) && o == (match payload(i) { Some(v) => v, None => 7u64 }, 9u64)
}
pub open spec fn model_initial(s: nat) -> bool { s == 0 }
pub open spec fn model_step(s: nat, i: Option<Delivery<u64>>, t: nat, o: (u64, u64)) -> bool {
    t == (if s < 100 { s + 1 } else { s }) && o == (match payload(i) { Some(v) => v, None => 7u64 }, 9u64)
}
pub open spec fn view(s: u64) -> nat { s as nat }
pub open spec fn bounded(s: u64, sent: Seq<Envelope<u64>>) -> bool { s <= 100 }

pub fn new_counter() -> (s: u64) ensures initial(s) { 0 }

// Deliberately disagrees with a possible application expectation that both
// recipients receive the same value. Authenticity must not conceal this bug.
pub fn handle(s: u64, input: Option<u64>) -> (r: (u64, (u64, u64)))
    ensures forall|i: Option<Delivery<u64>>| #[trigger] payload(i) == input ==> step(s, i, r.0, r.1),
{
    let t = if s < 100 { s + 1 } else { s };
    let first = match input { Some(value) => value, None => 7 };
    (t, (first, 9))
}

pub proof fn network_examples() {
    let a = message(1, 7);
    let b = message(2, 9);
    let empty = Seq::<Envelope<u64>>::empty();
    assert(!delivered(empty, Delivery { send: 0, message: a }));
    let sent = emitted((7, 9));
    assert(exchange(empty, None, sent, sent));
    assert(exchange(sent, Some(Delivery { send: 1, message: b }), empty, sent));
    // Delay the first message; deliver the second again, without consuming it.
    assert(exchange(sent, Some(Delivery { send: 1, message: b }), empty, sent));
    assert(exchange(sent, Some(Delivery { send: 0, message: a }), empty, sent));
    // A crash/no-delivery step does not erase provenance; loss is non-delivery.
    assert(exchange(sent, None, empty, sent));
    assert(!delivered(sent, Delivery { send: 0, message: message(1, 9) }));
    assert(!delivered(sent, Delivery { send: 0, message: message(2, 7) }));
    assert(!delivered(sent, Delivery { send: 0, message: Envelope { group: 2, ..a } }));
    assert(!delivered(sent, Delivery { send: 0, message: Envelope { sender: 2, ..a } }));
    let h = History { states: seq![(), (), (), (), (), ()],
        inputs: seq![None, Some(Delivery { send: 1, message: b }),
            Some(Delivery { send: 1, message: b }), Some(Delivery { send: 0, message: a }), None],
        outputs: seq![sent, empty, empty, empty, empty],
        sent: seq![empty, sent, sent, sent, sent, sent] };
    assert forall|k: int| 0 <= k < h.inputs.len() implies
        exchange(h.sent[k], #[trigger] h.inputs[k], h.outputs[k], h.sent[k+1]) by {
        assert(k == 0 || k == 1 || k == 2 || k == 3 || k == 4);
    }
    assert(valid(h, |s: ()| true, |s: (), i: Option<Delivery<u64>>, t: (), o: Seq<Envelope<u64>>| true,
        |i| i, |o| o));
}

pub fn actual_bug_is_admitted() {
    let (s, o) = handle(new_counter(), None);
    proof {
        assert(payload(None) == None::<u64>);
        assert(step(0, None, s, o));
        let sent = emitted(o);
        assert(o.0 != o.1);
        assert(delivered(sent, Delivery { send: 0, message: message(1, o.0) }));
        assert(delivered(sent, Delivery { send: 1, message: message(2, o.1) }));
    }
}

pub proof fn complete_toy_history(h: History<u64, Option<Delivery<u64>>, (u64, u64), u64>, k: int)
    requires valid(h, |s| initial(s), |s, i, t, o| step(s, i, t, o), |i| receive(i), |o| emitted(o)), 0 <= k < h.states.len(),
    ensures bounded(h.states[k], h.sent[k]),
        valid(project(h, |s| view(s)), |s| model_initial(s), |s, i, t, o| model_step(s, i, t, o), |i| receive(i), |o| emitted(o)),
{
    assert forall|s: u64| initial(s) implies #[trigger] bounded(s, Seq::empty()) && model_initial(view(s)) by {}
    assert forall|s: u64, i: Option<Delivery<u64>>, t: u64, o: (u64, u64), a: Seq<Envelope<u64>>, b: Seq<Envelope<u64>>|
        #[trigger] bounded(s, a) && #[trigger] step(s, i, t, o) && #[trigger] exchange(a, receive(i), emitted(o), b)
        implies bounded(t, b) && model_step(view(s), i, view(t), o) by {}
    invariant_at(h, |s| initial(s), |s, i, t, o| step(s, i, t, o), |i| receive(i), |o| emitted(o), |s, sent| bounded(s, sent), k);
    simulate(h, |s| initial(s), |s, i, t, o| step(s, i, t, o), |s| model_initial(s), |s, i, t, o| model_step(s, i, t, o), |i| receive(i), |o| emitted(o), |s| view(s), |s, sent| bounded(s, sent));
}
} // verus!
