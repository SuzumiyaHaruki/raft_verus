//! Independent calls of the very same Core used by Raft::request_vote.
#[path = "api.rs"] mod vote_api;
#[path = "../../madraft/src/raft/vote_core.rs"] mod vote_core;
#[cfg(any(rv_history, rv_confirmed, rv_pending))]
#[path = "../../madraft/proof/vote_lemmas.rs"] mod vote_lemmas;
use vote_api::*;
use vote_core::Core;
use vstd::prelude::*;
verus! {
// No request-validity precondition: invalid external inputs are checked by Core.
pub fn rules(c: &mut Core, r: Request, n: Completion) {
    proof { c.domain(); }
    let ghost before = c.view();
    let ghost outputs = c.replies();
    let ghost base = c.base();
    let out = c.receive(r);
    assert(out == receive_result(before,r));
    assert(c.view() == receive_after(before,r,out));
    assert(c.replies() == outputs && c.base() == base);
    let ghost pending = c.view();
    let reply = c.complete(n);
    assert(reply == complete_result(pending,n));
    assert(c.view() == complete_after(pending,n));
    assert(c.replies() == append_reply(outputs,reply) && c.base() == base);
    let actual = c.inspect();
    assert(actual == c.view());
    proof { c.domain(); }
}
pub fn constructors(c: Config, s: Initial, r: Record) {
    let fresh = Core::new(c,s);
    assert(fresh.is_some() <==> valid_initial(c,s));
    if let Some(f) = fresh {
        assert(f.view() == fresh_view(c,s));
        assert(f.replies() == Seq::<Reply>::empty() && f.base().is_none());
    }
    let recovered = Core::recover(c,r);
    assert(recovered.is_some() <==> valid_record(c,r));
    if let Some(a) = recovered {
        assert(a.view() == recovered_view(c,r));
        assert(a.replies() == Seq::<Reply>::empty() && a.base() == Some(r));
    }
}
// A real nonempty grant: always-deny and never-reply cannot satisfy this entry.
pub fn nonempty() {
    let mut c = Core::new(Config{node:0,members:3}, Initial{
        state:Persistent{term:2,voted_for:None,log:Log{term:1,index:4}},role:Role::Follower}).unwrap();
    let out = c.receive(Request{term:2,candidate:1,log:Log{term:1,index:4}});
    let Receive::Save(w) = out else { unreached() };
    assert(w.reply.granted && w.reply.candidate == 1);
    let reply = c.complete(Completion{op:w.record.op,success:true});
    assert(reply.is_some() && reply.unwrap().granted && reply.unwrap().candidate == 1);
    assert(c.replies().len() == 1 && c.replies()[0] == reply.unwrap());
}

#[cfg(any(rv_history, rv_confirmed, rv_pending))]
pub fn finite_calls(c: &mut Core, events: &Vec<Event>) -> (observed: Ghost<Seq<Reply>>)
    ensures observed@ == c.replies(), unique(observed@),
        c.base() == old(c).base(), c.view().config == old(c).view().config,
{
    let ghost base = c.base();
    let ghost config = c.view().config;
    let ghost mut outputs = c.replies();
    let mut i: usize = 0;
    while i < events.len()
        invariant i <= events.len(), outputs == c.replies(),
            c.base() == base, c.view().config == config,
        decreases events.len() - i,
    {
        match events[i] {
            Event::Receive(r) => { c.receive(r); },
            Event::Complete(n) => {
                let actual_reply = c.complete(n);
                proof { outputs = append_reply(outputs,actual_reply); }
            },
        }
        i += 1;
    }
    proof { vote_lemmas::uniqueness(c); }
    Ghost(outputs)
}
#[cfg(rv_history)]
pub fn from_constructor(config: Config, initial: Initial, events: &Vec<Event>)
    requires valid_initial(config,initial),
{
    let mut c = Core::new(config,initial).unwrap();
    assert(c.replies() == Seq::<Reply>::empty());
    let outputs = finite_calls(&mut c, events);
    assert(unique(outputs@));
}

// readback == the actual emitted Write is an explicit faithful-I/O premise.
// It is NOT proved by a completion boolean. Each caller below calls receive,
// then recover, and drives an arbitrary suffix of real calls.
#[cfg(any(rv_confirmed, rv_pending))]
pub fn restart_window(before: &mut Core, request: Request, readback: Record,
    suffix: &Vec<Event>, confirmed: bool)
    requires receive_result(old(before).view(),request) is Save,
        receive_result(old(before).view(),request)->Save_0.record == readback,
        receive_result(old(before).view(),request)->Save_0.reply.granted,
        old(before).view().next < u64::MAX,
{
    let ghost old_outputs = before.replies();
    let Receive::Save(w) = before.receive(request) else { unreached() };
    proof { before.domain(); }
    assert(w.record == readback); // not a fabricated old complete/reply
    assert(valid_record(before.view().config,readback));
    if confirmed {
        let actual = before.complete(Completion{op:w.record.op,success:true});
        assert(actual == Some(w.reply) && actual.unwrap().granted);
    } else {
        assert(before.replies() == old_outputs);
        assert(before.view().pending == Some(w));
    }
    let config = before.inspect().config;
    let mut after = Core::recover(config,readback).unwrap();
    assert(after.view() == recovered_view(config,readback));
    assert(after.replies() == Seq::<Reply>::empty()); // no automatic old reply
    assert(after.view().state.voted_for == Some(request.candidate));
    let observed = finite_calls(&mut after,suffix);
    proof { vote_lemmas::restart_composition(before,&after,readback); }
    assert(unique(before.replies() + observed@));
}
#[cfg(rv_confirmed)]
pub fn confirmed(before: &mut Core, r: Request, disk: Record, suffix: &Vec<Event>)
    requires receive_result(old(before).view(),r) is Save,
        receive_result(old(before).view(),r)->Save_0.record == disk,
        receive_result(old(before).view(),r)->Save_0.reply.granted,
        old(before).view().next < u64::MAX,
{ restart_window(before,r,disk,suffix,true); }
#[cfg(rv_pending)]
pub fn written_pending(before: &mut Core, r: Request, disk: Record, suffix: &Vec<Event>)
    requires receive_result(old(before).view(),r) is Save,
        receive_result(old(before).view(),r)->Save_0.record == disk,
        receive_result(old(before).view(),r)->Save_0.reply.granted,
        old(before).view().next < u64::MAX,
{ restart_window(before,r,disk,suffix,false); }
}
