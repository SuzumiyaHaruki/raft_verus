//! Fixed observer of actual executable calls, not a candidate-selected trace.
#[path="api.rs"] mod vote_api;
#[path="../../madraft/src/raft/vote_core.rs"] mod vote_core;
#[path="../../madraft/proof/vote_lemmas.rs"] mod vote_lemmas;
#[cfg(any(es_provenance,es_nonempty,es_election_basis,es_election_history,es_election_nonempty,es_restart_source,es_restart_history,es_restart_nonempty))]
#[path="network.rs"] mod vote_network;
#[path="observation.rs"] mod election_observation;
#[path="../../requirements/targets.rs"] mod targets;
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
            Event::Promote => { c.try_become_leader(); },
            Event::Start => {
                let actual = c.start_election();
                if let Campaign::Save(_) = actual {
                    let state = c.inspect();
                    assert(state.state.term == before.state.term + 1);
                    proof { h = h.push(choice(state)); }
                }
            },
            Event::Receive(r) => { c.receive(r); },
            Event::Rpc(context,result) => { c.collect(context,result); },
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
    assert(after.votes() == Set::<u64>::empty() && after.view().round.is_none() && !after.view().self_saved);
    assert(after.view().role == Role::Follower && after.view().pending.is_none());
    assert(after.view().state.voted_for == before.view().state.voted_for);
    // Old completion cannot authorize an old action or consume a new ID.
    let stale = after.complete(Completion{op:w.record.op,success:true}); assert(stale.is_none());
    let old_context=RpcContext{sent:Send{config,request:Request{term:readback.state.term,candidate:config.node,log:readback.state.log}},peer:if config.node==0 {1} else {0}};
    after.collect(old_context,RpcResult::Reply{term:readback.state.term,granted:true});
    assert(after.votes()==Set::<u64>::empty() && after.view().round.is_none());
    let elected=after.try_become_leader();assert(!elected && after.view().role==Role::Follower);
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

#[cfg(es_reply_rules)]
pub fn reply_rules(c:&mut Core, context:RpcContext, result:RpcResult, member:u64) {
    let ghost before=c.view();let ghost votes=c.votes();let ghost history=c.choices();
    let actual=c.collect(context,result);
    assert(actual==collect_result(before,votes,context,result));
    assert(c.view()==collect_after(before,context,result,actual));
    assert(c.votes()==collect_votes(before,votes,context,result,actual));
    assert(c.choices()==history && c.view().state.term>=before.state.term);
    proof { c.domain(); }
    let count=c.vote_count();let included=c.has_vote(member);
    assert(count as nat==c.votes().len() && count<=c.view().config.members);
    assert(included==c.votes().contains(member));
}
#[cfg(es_self_vote)]
pub fn saved_self(c:&mut Core) {
    let out=c.start_election();
    if let Campaign::Save(w)=out {
        assert(c.votes()==Set::<u64>::empty() && !c.view().self_saved);
        let action=c.complete(Completion{op:w.record.op,success:true});
        assert(action==Some(w.action));
        assert(c.votes()==Set::<u64>::empty().insert(c.view().config.node));
        assert(c.view().self_saved);
        let count=c.vote_count();assert(count==1);
        let again=c.complete(Completion{op:w.record.op,success:true});
        assert(again.is_none());let count=c.vote_count();assert(count==1);
    }
}
#[cfg(es_rounds)]
pub fn dedup_and_old_round(c:&mut Core, context:RpcContext, result:RpcResult)
    requires !higher(old(c).view(),context,result),
{
    let ghost before=c.votes();let ghost view=c.view();
    let first=c.collect(context,result);let ghost votes=c.votes();
    let second=c.collect(context,result);
    assert(c.votes()==votes);
    assert(first==Collect::Counted ==> votes==before.insert(context.peer) && !before.contains(context.peer));
    assert(context.sent.request.term!=view.state.term ==> c.votes()==before);
}
#[cfg(es_higher_reply)]
pub fn higher_old_rpc(c:&mut Core, context:RpcContext, term:u64, granted:bool)
    requires !old(c).view().halted, old(c).view().pending.is_none(), valid_context(old(c).view(),context), term>old(c).view().state.term,
{
    let ghost before=c.view();
    let out=c.collect(context,RpcResult::Reply{term,granted});
    assert(c.view().state.term==term && c.view().role==Role::Follower && c.view().state.voted_for.is_none());
    assert(c.votes()==Set::<u64>::empty() && c.view().round.is_none());
    if let Collect::Save(w)=out {
        assert(w.request.is_none() && w.action==Action::Persisted);
        let action=c.complete(Completion{op:w.record.op,success:true});
        assert(action==Some(Action::Persisted));
    } else { assert(before.next==u64::MAX && c.view().halted); }
}
#[cfg(es_provenance)]
pub fn all_finite_messages(initials:&Vec<Initial>, events:&Vec<vote_network::Step>, query_node:usize, member:u64)
    requires 0<initials.len()<=u64::MAX, query_node<initials.len(),
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let mut network=vote_network::new_network(initials);
    proof { vote_lemmas::network_initial(network.trace()); vote_lemmas::network_conclusion(network.trace()); }
    let mut k:usize=0;
    while k<events.len()
        invariant k<=events.len(), query_node<network.nodes.len(), vote_network::shape(network.trace()), vote_lemmas::network_inv(network.trace()),
        decreases events.len()-k,
    {
        // Only bounds/routing/pending-I/O. No term, grant, invariant or safety filtering.
        let event=events[k];
        if !vote_network::is_enabled(&network,event) { return; }
        let ghost before=network.trace();
        vote_network::execute(&mut network,event);
        proof {
            vote_lemmas::network_step(before,network.trace(),event);
            vote_lemmas::network_conclusion(network.trace());
        }
        assert(vote_network::sourced(network.trace()));
        let (count,has)=vote_network::query(&network,query_node,member);
        assert(count as nat==network.trace().nodes[query_node as int].votes.len());
        assert(has==network.trace().nodes[query_node as int].votes.contains(member));
        k+=1;
    }
}
#[cfg(es_nonempty)]
pub fn three_real_receivers(duplicate_only:bool) {
    let initial=Initial{state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower};
    let mut a=Core::new(Config{node:0,members:3},initial).unwrap();
    let mut b=Core::new(Config{node:1,members:3},initial).unwrap();
    let mut c=Core::new(Config{node:2,members:3},initial).unwrap();
    let Campaign::Save(w)=a.start_election() else { unreached() };
    let Some(Action::Send(sent))=a.complete(Completion{op:w.record.op,success:true}) else { unreached() };
    let Receive::Save(w)=b.receive(sent.request) else { unreached() };
    let Some(Action::Reply(br))=b.complete(Completion{op:w.record.op,success:true}) else { unreached() };
    let context=RpcContext{sent,peer:1};
    a.collect(context,RpcResult::Reply{term:br.term,granted:br.granted});
    a.collect(context,RpcResult::Reply{term:br.term,granted:br.granted});
    let count=a.vote_count();assert(count==2);
    if !duplicate_only {
        let Receive::Save(w)=c.receive(sent.request) else { unreached() };
        let Some(Action::Reply(cr))=c.complete(Completion{op:w.record.op,success:true}) else { unreached() };
        a.collect(RpcContext{sent,peer:2},RpcResult::Reply{term:cr.term,granted:cr.granted});
        let count=a.vote_count();assert(count==3);
        let a0=a.has_vote(0);let a1=a.has_vote(1);let a2=a.has_vote(2);assert(a0 && a1 && a2);
    }
    assert(a.view().role==Role::Candidate);
}

#[cfg(es_promotion)]
pub fn promotion(c:&mut Core) {
    proof { c.domain(); }
    let old_view=c.inspect();let ghost before=c.view();let ghost votes=c.votes();let ghost h=c.choices();
    let elected=c.try_become_leader();let actual=c.inspect();
    assert(elected==promotion_allowed(before,votes));
    assert(actual==promotion_after(before,votes));
    assert(actual.state==before.state && actual.next==before.next && actual.pending==before.pending);
    assert(c.votes()==votes && c.choices()==h);
    let change=election_observation::Change{node:actual.config.node,before:old_view,after:actual};
    // The observer reads role changes, never the returned flag as its source.
    assert(election_observation::elects(change)==elected);
    let second=c.try_become_leader();assert(!second && c.view()==actual);
}
#[cfg(any(es_election_basis,es_election_history))]
pub fn finite_elections(initials:&Vec<Initial>, events:&Vec<vote_network::Step>, group:Ghost<targets::Group>, h:Ghost<targets::History>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let mut n=vote_network::new_network(initials);
    proof { vote_lemmas::network_initial(n.trace()); vote_lemmas::network_conclusion(n.trace()); }
    let mut k:usize=0;
    while k<events.len()
        invariant k<=events.len(), vote_network::shape(n.trace()),vote_lemmas::network_inv(n.trace()),n.trace().frames.len()==k,
        decreases events.len()-k,
    {
        let e=events[k];if !vote_network::is_enabled(&n,e) { return; }
        let ghost before=n.trace();vote_network::execute(&mut n,e);
        assert(n.trace().frames.len()==before.frames.len()+1);
        assert(n.trace().frames.drop_last()==before.frames);
        let ghost f=n.trace().frames.last();
        assert(f.change.before==before.nodes[vote_network::actor(before,e)].state);
        assert(f.change.after==n.trace().nodes[vote_network::actor(before,e)].state);
        proof { vote_lemmas::network_step(before,n.trace(),e);vote_lemmas::network_conclusion(n.trace()); }
        assert(vote_network::sourced(n.trace()) && vote_network::choice_sourced(n.trace()));
        assert(vote_network::elections_justified(n.trace()) && vote_network::historical_safety(n.trace()));
        // Any History with these actual events satisfies the existing target;
        // no unrelated log or snapshot state is manufactured here.
        proof { if h@.events==election_observation::projection(vote_network::changes(n.trace()),group@) {
            election_observation::bridge(vote_network::changes(n.trace()),group@,h@);
            assert(targets::election_safety(h@));
        } }
        k+=1;
    }
}
#[cfg(es_election_nonempty)]
pub fn elected_witness(single:bool) {
    let initial=Initial{state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower};
    let members=if single {1} else {3};
    let mut a=Core::new(Config{node:0,members},initial).unwrap();
    let Campaign::Save(w)=a.start_election() else {unreached()};
    let premature=a.try_become_leader();assert(!premature && a.view().role==Role::Candidate);
    let Some(Action::Send(sent))=a.complete(Completion{op:w.record.op,success:true}) else {unreached()};
    if !single {
        let short=a.try_become_leader();assert(!short);
        let mut b=Core::new(Config{node:1,members},initial).unwrap();
        let mut c=Core::new(Config{node:2,members},initial).unwrap();
        let Receive::Save(w)=b.receive(sent.request) else {unreached()};
        let Some(Action::Reply(r))=b.complete(Completion{op:w.record.op,success:true}) else {unreached()};
        a.collect(RpcContext{sent,peer:1},RpcResult::Reply{term:r.term,granted:r.granted});
        let count=a.vote_count();assert(count==2);assert(c.view().role==Role::Follower);
    }
    let before=a.inspect();let elected=a.try_become_leader();let after=a.inspect();
    assert(elected && after.role==Role::Leader && after.state.term==1);
    let actual=election_observation::observe(election_observation::Change{node:0,before,after});
    assert(actual==Some(election_observation::Election{node:0,term:1}));
    let duplicate=a.try_become_leader();assert(!duplicate && a.view()==after);
}

}

#[cfg(any(es_restart_source,es_restart_history,es_restart_nonempty))]
#[path="recovery_network.rs"] mod recovery_network;
#[cfg(any(es_restart_source,es_restart_history))]
#[path="../../madraft/proof/recovery_lemmas.rs"] mod recovery_lemmas;
#[cfg(any(es_restart_source,es_restart_history))]
verus! {
pub fn finite_restarts(initials:&Vec<Initial>,events:&Vec<recovery_network::Step>,group:Ghost<targets::Group>,h:Ghost<targets::History>)
    requires 0<initials.len()<=u64::MAX,
        forall|i:int| 0<=i<initials.len() ==> valid_initial(Config{node:i as u64,members:initials.len() as u64},#[trigger] initials[i]) && initials[i].role==Role::Follower,
{
    let mut n=recovery_network::new_network(initials);
    proof{recovery_lemmas::initialize(n.trace());recovery_lemmas::conclude(n.trace());}
    let mut k:usize=0;
    while k<events.len()
        invariant k<=events.len(),n.represented(),n.trace().frames.len()==k,recovery_lemmas::recovery_inv(n.trace()),
        decreases events.len()-k,
    {
        let s=events[k];if !recovery_network::is_enabled(&n,s) {return;}
        let ghost before=n.trace();recovery_network::execute(&mut n,s);
        assert(n.trace().frames.drop_last()==before.frames);
        assert(before.choices.is_prefix_of(n.trace().choices));
        proof{recovery_lemmas::preserve(before,n.trace(),s);recovery_lemmas::conclude(n.trace());}
        assert(recovery_network::recovered_from_own_write(n.trace()));
        assert(unique(n.trace().choices));
        assert(recovery_network::provenance(n.trace()));
        assert(recovery_network::historical_safety(n.trace()));
        proof{if h@.events==recovery_network::projection(n.trace(),group@) {
            recovery_network::bridge(n.trace(),group@,h@);assert(targets::election_safety(h@));
        }}
        k+=1;
    }
}
}
#[cfg(es_restart_nonempty)]
verus! {
// Nonempty environment windows, independently of candidate-selected invariants.
#[verifier::spinoff_prover]
pub fn environment_window(before_reply:bool) {
    use recovery_network::{Step as R,execute as step};
    let initial=Initial{state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower};
    let mut initials=Vec::new();initials.push(initial);initials.push(initial);initials.push(initial);
    let mut n=recovery_network::new_network(&initials);
    step(&mut n,R::Start(0));step(&mut n,R::Write{node:0,success:true});
    if !before_reply {step(&mut n,R::Notify(0));}
    let ghost sent=n.trace().net.sent;
    step(&mut n,R::Stop(0));step(&mut n,R::Restart(0));
    assert(n.trace().live[0] && n.trace().epochs[0]==1);
    assert(n.trace().net.nodes[0].state.state.voted_for==Some(0u64));
    assert(n.trace().net.nodes[0].state.role==Role::Follower);
    assert(n.trace().net.sent==sent && n.trace().net.nodes[0].votes==Set::<u64>::empty());
}
// A concrete actual-call witness, not a supplied implementation or invariant.
// Records below come only from the corresponding Core's actual Write. Success
// denotes the disclosed complete-write premise, not a filesystem proof.
#[verifier::spinoff_prover]
pub fn restart_witness(before_reply:bool) {
    let initial=Initial{state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower};
    let ca=Config{node:0,members:3};let cb=Config{node:1,members:3};let cc=Config{node:2,members:3};
    let mut a=Core::new(ca,initial).unwrap();let mut b=Core::new(cb,initial).unwrap();let mut c=Core::new(cc,initial).unwrap();
    let Campaign::Save(aw)=a.start_election() else {unreached()};
    let Some(Action::Send(asent))=a.complete(Completion{op:aw.record.op,success:true}) else {unreached()};
    let Campaign::Save(cw)=c.start_election() else {unreached()};
    let Some(Action::Send(csent))=c.complete(Completion{op:cw.record.op,success:true}) else {unreached()};
    let Receive::Save(bw)=b.receive(asent.request) else {unreached()};
    if !before_reply {b.complete(Completion{op:bw.record.op,success:true});}
    // Discard the instance. There is deliberately no complete in window B.
    b=Core::recover(cb,bw.record).unwrap();
    assert(b.view().state.voted_for==Some(0u64) && b.votes()==Set::<u64>::empty());
    let Receive::Save(denied)=b.receive(csent.request) else {unreached()};
    let Some(Action::Reply(no))=b.complete(Completion{op:denied.record.op,success:true}) else {unreached()};
    assert(!no.granted);
    let Receive::Save(retry)=b.receive(asent.request) else {unreached()};
    let Some(Action::Reply(yes))=b.complete(Completion{op:retry.record.op,success:true}) else {unreached()};
    a.collect(RpcContext{sent:asent,peer:1},RpcResult::Reply{term:yes.term,granted:yes.granted});
    let elected=a.try_become_leader();assert(elected && a.view().role==Role::Leader);
    b=Core::recover(cb,retry.record).unwrap(); // Second restart of the voter.
    a=Core::recover(ca,aw.record).unwrap(); // A different node, previously elected.
    assert(a.view().role==Role::Follower && a.votes()==Set::<u64>::empty());
    let Campaign::Save(next)=a.start_election() else {unreached()};
    let Some(Action::Send(sent))=a.complete(Completion{op:next.record.op,success:true}) else {unreached()};
    let Receive::Save(vote)=b.receive(sent.request) else {unreached()};
    let Some(Action::Reply(yes))=b.complete(Completion{op:vote.record.op,success:true}) else {unreached()};
    a.collect(RpcContext{sent,peer:1},RpcResult::Reply{term:yes.term,granted:yes.granted});
    let elected=a.try_become_leader();assert(elected && a.view().state.term==2);
}
}
