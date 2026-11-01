//! Fixed observer of actual executable calls, not a candidate-selected trace.
#[path="api.rs"] mod vote_api;
#[path="../../madraft/src/raft/vote_core.rs"] mod vote_core;
#[path="../../madraft/proof/vote_lemmas.rs"] mod vote_lemmas;
use vote_api::*;
use vote_core::Core;
use vstd::prelude::*;
verus! {
pub fn constructors(c: Config, s: Initial, r: Record) {
    let fresh = Core::new(c,s);
    assert(fresh.is_some() <==> valid_initial(c,s));
    if let Some(f) = fresh {
        assert(f.view() == fresh_view(c,s));
        assert(f.choices() == Seq::<Choice>::empty() && f.base().is_none());
    }
    let recovered = Core::recover(c,r);
    assert(recovered.is_some() <==> valid_record(c,r));
    if let Some(a) = recovered {
        assert(a.view() == recovered_view(c,r));
        assert(a.choices() == Seq::<Choice>::empty() && a.base() == Some(r));
    }
}
pub fn receiver_rules(c: &mut Core, r: Request, n: Completion) {
    proof { c.domain(); }
    let ghost before = c.view();
    let ghost h = c.choices();
    let out = c.receive(r);
    assert(out == receive_result(before,r));
    assert(c.view() == receive_after(before,r,out));
    assert(c.view().state.term >= before.state.term);
    assert(c.choices() == h);
    let ghost pending = c.view();
    let action = c.complete(n);
    assert(action == complete_result(pending,n));
    assert(c.view() == complete_after(pending,n));
    assert(c.choices() == append_action(h,action));
    let actual = c.inspect();
    assert(actual == c.view());
}
pub fn receiver_nonempty() {
    let mut c = Core::new(Config{node:0,members:3},Initial{state:Persistent{
        term:2,voted_for:None,log:Log{term:1,index:4}},role:Role::Follower}).unwrap();
    let Receive::Save(w) = c.receive(Request{term:2,candidate:1,log:Log{term:1,index:4}}) else { unreached() };
    let action = c.complete(Completion{op:w.record.op,success:true});
    assert(action.is_some() && action.unwrap() is Reply);
    let Action::Reply(r) = action.unwrap() else { unreached() };
    assert(r.granted && r.candidate == 1);
    assert(c.choices() == seq![Choice{node:0,term:2,candidate:1}]);
}
#[cfg(es_campaign)]
pub fn campaign_rules(c: &mut Core) {
    proof { c.domain(); }
    let ghost before = c.view();
    let ghost h = c.choices();
    let out = c.start_election();
    let actual = c.inspect();
    assert(out == start_result(before));
    assert(actual == start_after(before,out));
    match out {
        Campaign::Save(w) => {
            assert(actual.state.term == before.state.term + 1);
            assert(actual.state.voted_for == Some(actual.config.node));
            assert(actual.state.log == before.state.log && actual.role == Role::Candidate);
            assert(c.choices() == h.push(choice(actual)));
            assert(actual.pending == Some(w));
        },
        Campaign::Reject(_) => { assert(actual == before && c.choices() == h); },
    }
}
#[cfg(any(es_campaign,es_authorization))]
pub fn campaign_nonempty() {
    let mut c = Core::new(Config{node:0,members:3},Initial{state:Persistent{
        term:2,voted_for:Some(1),log:Log{term:1,index:4}},role:Role::Follower}).unwrap();
    let Campaign::Save(w) = c.start_election() else { unreached() };
    assert(c.view().state.term == 3 && c.view().state.voted_for == Some(0u64));
    assert(c.choices() == seq![Choice{node:0,term:3,candidate:0}]);
    let wrong = c.complete(Completion{op:1,success:true}); assert(wrong.is_none());
    let actual = c.complete(Completion{op:w.record.op,success:true});
    assert(actual == Some(Action::Send(Send{config:Config{node:0,members:3},
        request:Request{term:3,candidate:0,log:Log{term:1,index:4}}})));
    let duplicate = c.complete(Completion{op:w.record.op,success:true}); assert(duplicate.is_none());
}
#[cfg(es_authorization)]
pub fn completion_authority(c: &mut Core, n: Completion) {
    let ghost v = c.view();
    let actual = c.complete(n);
    assert(actual.is_some() <==> matches(v,n) && n.success);
    if let Some(a) = actual {
        assert(a == v.pending.unwrap().action);
        assert(n.op == v.pending.unwrap().record.op && n.success);
    } else { proof { if !matches(v,n) { assert(c.view() == v); } } }
}
// This loop independently observes the actual state after accepted Start and
// actual affirmative return values. A self-choice is not I/O or a counted vote.
#[cfg(any(es_history,es_receive_confirmed,es_receive_pending,es_self_confirmed,es_self_pending))]
pub fn mixed_calls(c: &mut Core, events: &Vec<Event>) -> (observed: Ghost<Seq<Choice>>)
    ensures observed@ == c.choices(), unique(observed@),
        c.base() == old(c).base(), c.view().config == old(c).view().config,
        c.view().state.term >= old(c).view().state.term,
{
    let ghost base = c.base(); let ghost config = c.view().config;
    let ghost term = c.view().state.term;
    let ghost mut h = c.choices();
    let mut i: usize = 0;
    while i < events.len()
        invariant i <= events.len(), h == c.choices(), c.base() == base,
            c.view().config == config, c.view().state.term >= term,
        decreases events.len() - i,
    {
        let ghost before = c.view();
        match events[i] {
            Event::Start => {
                let actual = c.start_election();
                if let Campaign::Save(_) = actual {
                    let state = c.inspect();
                    assert(state.state.term == before.state.term + 1);
                    proof { h = h.push(choice(state)); }
                }
            },
            Event::Receive(r) => { c.receive(r); },
            Event::Complete(n) => {
                let actual = c.complete(n);
                assert(actual == complete_result(before,n));
                proof { h = append_action(h,actual); }
            },
        }
        assert(c.view().state.term >= before.state.term);
        i += 1;
    }
    proof { vote_lemmas::uniqueness(c); }
    Ghost(h)
}
#[cfg(es_history)]
pub fn from_constructor(config: Config, initial: Initial, events: &Vec<Event>)
    requires valid_initial(config,initial),
{
    let mut c = Core::new(config,initial).unwrap();
    assert(c.choices() == Seq::<Choice>::empty());
    let actual = mixed_calls(&mut c,events);
    assert(unique(actual@));
}
// Equality to the actual emitted record is the public faithful-I/O premise.
// Neither uniqueness nor correct recovery is assumed. No fake old completion.
#[cfg(any(es_receive_confirmed,es_receive_pending,es_self_confirmed,es_self_pending))]
pub fn restart_window(config: Config, initial: Initial, prefix: &Vec<Event>, r: Request,
    readback: Record, suffix: &Vec<Event>, campaign: bool, completed: bool)
    requires valid_initial(config,initial),
{
    let mut before = Core::new(config,initial).unwrap();
    let prefix_observed = mixed_calls(&mut before,prefix);
    assert(prefix_observed@ == before.choices());
    let ghost old_choices = before.choices();
    let w = if campaign {
        let Campaign::Save(w) = before.start_election() else { return }; w
    } else {
        let Receive::Save(w) = before.receive(r) else { return }; w
    };
    // Reject an external readback outside the two specified faithful windows.
    // This guard is NOT a filter on voting correctness or history uniqueness.
    if readback.op != w.record.op || readback.state.term != w.record.state.term
        || readback.state.voted_for != w.record.state.voted_for
        || readback.state.log.term != w.record.state.log.term
        || readback.state.log.index != w.record.state.log.index { return; }
    if !campaign { match w.action { Action::Reply(reply) => { if !reply.granted { return; } }, _ => { unreached() } } }
    proof { before.domain(); }
    assert(w.record == readback && valid_record(before.view().config,readback));
    if campaign { assert(before.choices() == old_choices.push(choice(before.view()))); }
    if completed {
        let actual = before.complete(Completion{op:w.record.op,success:true});
        assert(actual == Some(w.action));
    } else {
        assert(before.view().pending == Some(w));
        if !campaign { assert(before.choices() == old_choices); }
    }
    let config = before.inspect().config;
    let mut after = Core::recover(config,readback).unwrap();
    assert(after.view() == recovered_view(config,readback));
    assert(after.choices() == Seq::<Choice>::empty());
    assert(after.view().role == Role::Follower && after.view().pending.is_none());
    assert(after.view().state.voted_for == before.view().state.voted_for);
    // Old completion cannot authorize an old action or consume a new ID.
    let stale = after.complete(Completion{op:w.record.op,success:true}); assert(stale.is_none());
    let actual = mixed_calls(&mut after,suffix);
    proof { vote_lemmas::restart_composition(&before,&after,readback); }
    assert(unique(before.choices() + actual@));
}
#[cfg(es_receive_confirmed)]
pub fn receive_confirmed(config: Config, initial: Initial, prefix: &Vec<Event>, r: Request, disk: Record, suffix: &Vec<Event>)
    requires valid_initial(config,initial),
{ restart_window(config,initial,prefix,r,disk,suffix,false,true); }

#[cfg(es_receive_pending)]
pub fn receive_pending(config: Config, initial: Initial, prefix: &Vec<Event>, r: Request, disk: Record, suffix: &Vec<Event>)
    requires valid_initial(config,initial),
{ restart_window(config,initial,prefix,r,disk,suffix,false,false); }

#[cfg(es_self_confirmed)]
pub fn self_confirmed(config: Config, initial: Initial, prefix: &Vec<Event>, r: Request, disk: Record, suffix: &Vec<Event>)
    requires valid_initial(config,initial),
{ restart_window(config,initial,prefix,r,disk,suffix,true,true); }

#[cfg(es_self_pending)]
pub fn self_pending(config: Config, initial: Initial, prefix: &Vec<Event>, r: Request, disk: Record, suffix: &Vec<Event>)
    requires valid_initial(config,initial),
{ restart_window(config,initial,prefix,r,disk,suffix,true,false); }

}
