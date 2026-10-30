//! Protocol-independent finite histories. Ghost send indices are not wire fields.
use vstd::prelude::*;
verus! {
pub struct Envelope<M> { pub group: nat, pub sender: nat, pub recipient: nat, pub body: M }
pub struct Delivery<M> { pub send: nat, pub message: Envelope<M> }

pub open spec fn delivered<M>(sent: Seq<Envelope<M>>, d: Delivery<M>) -> bool {
    d.send < sent.len() && sent[d.send as int] == d.message
}

pub open spec fn exchange<M>(sent: Seq<Envelope<M>>, input: Option<Delivery<M>>,
    output: Seq<Envelope<M>>, next: Seq<Envelope<M>>) -> bool {
    (match input { Some(d) => delivered(sent, d), None => true }) && next == sent + output
}

// Adapter types, not a required executable architecture. A step may be a host
// operation, constructor, handler segment or other justified linearization.
pub struct History<S, I, O, M> {
    pub states: Seq<S>, pub inputs: Seq<I>, pub outputs: Seq<O>,
    pub sent: Seq<Seq<Envelope<M>>>,
}

pub open spec fn valid<S, I, O, M>(h: History<S, I, O, M>, initial: spec_fn(S) -> bool,
    step: spec_fn(S, I, S, O) -> bool, receive: spec_fn(I) -> Option<Delivery<M>>,
    emit: spec_fn(O) -> Seq<Envelope<M>>) -> bool {
    &&& h.states.len() == h.inputs.len() + 1
    &&& h.outputs.len() == h.inputs.len()
    &&& h.sent.len() == h.states.len()
    &&& h.sent[0] == Seq::<Envelope<M>>::empty()
    &&& initial(h.states[0])
    &&& forall|k: int| 0 <= k < h.inputs.len() ==> step(h.states[k], #[trigger] h.inputs[k], h.states[k+1], h.outputs[k])
        && exchange(h.sent[k], receive(h.inputs[k]), emit(h.outputs[k]), h.sent[k+1])
}

pub proof fn invariant_at<S, I, O, M>(h: History<S, I, O, M>, initial: spec_fn(S) -> bool,
    step: spec_fn(S, I, S, O) -> bool, receive: spec_fn(I) -> Option<Delivery<M>>,
    emit: spec_fn(O) -> Seq<Envelope<M>>, inv: spec_fn(S, Seq<Envelope<M>>) -> bool, k: int)
    requires valid(h, initial, step, receive, emit), 0 <= k < h.states.len(),
        forall|s: S| initial(s) ==> #[trigger] inv(s, Seq::empty()),
        forall|s: S, i: I, t: S, o: O, a: Seq<Envelope<M>>, b: Seq<Envelope<M>>|
            #[trigger] inv(s, a) && #[trigger] step(s, i, t, o) && #[trigger] exchange(a, receive(i), emit(o), b) ==> inv(t, b),
    ensures inv(h.states[k], h.sent[k]),
    decreases k,
{
    if k > 0 {
        invariant_at(h, initial, step, receive, emit, inv, k-1);
        assert(h.inputs.len() == h.states.len() - 1);
        assert(step(h.states[k-1], h.inputs[k-1], h.states[k], h.outputs[k-1]));
    }
}

pub open spec fn project<R, S, I, O, M>(h: History<R, I, O, M>, view: spec_fn(R) -> S) -> History<S, I, O, M> {
    History { states: Seq::new(h.states.len(), |k: int| view(h.states[k])),
        inputs: h.inputs, outputs: h.outputs, sent: h.sent }
}

// Coverage direction: every admitted executable history maps to a model history,
// preserving received inputs, actual output messages, and send provenance.
pub proof fn simulate<R, S, I, O, M>(h: History<R, I, O, M>, rinit: spec_fn(R) -> bool,
    rstep: spec_fn(R, I, R, O) -> bool, initial: spec_fn(S) -> bool,
    step: spec_fn(S, I, S, O) -> bool, receive: spec_fn(I) -> Option<Delivery<M>>,
    emit: spec_fn(O) -> Seq<Envelope<M>>, view: spec_fn(R) -> S,
    inv: spec_fn(R, Seq<Envelope<M>>) -> bool)
    requires valid(h, rinit, rstep, receive, emit),
        forall|r: R| rinit(r) ==> #[trigger] inv(r, Seq::empty()) && initial(view(r)),
        forall|r: R, i: I, t: R, o: O, a: Seq<Envelope<M>>, b: Seq<Envelope<M>>|
            #[trigger] inv(r, a) && #[trigger] rstep(r, i, t, o) && #[trigger] exchange(a, receive(i), emit(o), b)
                ==> inv(t, b) && step(view(r), i, view(t), o),
    ensures valid(project(h, view), initial, step, receive, emit),
{
    assert forall|r: R| rinit(r) implies #[trigger] inv(r, Seq::empty()) by {}
    assert forall|r: R, i: I, t: R, o: O, a: Seq<Envelope<M>>, b: Seq<Envelope<M>>|
        #[trigger] inv(r, a) && #[trigger] rstep(r, i, t, o) && #[trigger] exchange(a, receive(i), emit(o), b)
            implies inv(t, b) by {}
    invariant_at(h, rinit, rstep, receive, emit, inv, 0);
    assert(initial(view(h.states[0])));
    let p = project(h, view);
    assert forall|k: int| 0 <= k < p.inputs.len() implies step(p.states[k], #[trigger] p.inputs[k], p.states[k+1], p.outputs[k])
        && exchange(p.sent[k], receive(p.inputs[k]), emit(p.outputs[k]), p.sent[k+1]) by {
        invariant_at(h, rinit, rstep, receive, emit, inv, k);
        assert(rstep(h.states[k], h.inputs[k], h.states[k+1], h.outputs[k]));
    }
}
} // verus!
