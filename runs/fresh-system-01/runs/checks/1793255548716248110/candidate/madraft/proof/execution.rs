//! Developing cluster execution, including real log and storage effects.
#[path = "../src/raft/election.rs"]
pub mod election;
#[path = "../src/raft/replication.rs"]
pub mod replication;
use crate::targets;
use vstd::prelude::*;

verus! {
pub struct Snapshot { pub index:u64,pub term:u64,pub bytes:Seq<u8> }
pub struct Stored { pub base:u64,pub base_term:u64,pub prefix:Seq<replication::Entry>,pub snapshot:Option<Snapshot>,pub term: u64, pub vote: Option<usize>, pub entries: Seq<replication::Entry>, pub commit: u64 }
pub enum Disk { Missing, Incomplete, Complete(Stored) }
pub struct Node {
    pub election: election::Image, pub log: replication::Image,
    pub prefix:Seq<replication::Entry>,pub snapshot:Option<Snapshot>,
    pub disk: Disk, pub pending: Option<Stored>, pub alive: bool,
}
pub enum Input {
    Election(election::Input),
    Propose(replication::Entry),
    Append { index: u64, entry: replication::Entry },
    FollowerCommit { advertised: u64, matched: u64 },
    LeaderCommit { matched: Seq<u64> },
    Apply { delivered: bool },
    Compact{index:u64,bytes:Seq<u8>},
    Install{index:u64,term:u64,bytes:Seq<u8>,prefix:Seq<replication::Entry>},
    DeliverSnapshot,
    BeginWrite,
    CompleteWrite,
    Crash,
    Restart,
}
pub struct Action { pub who: targets::Key, pub input: Input }
pub struct Execution { pub states: Seq<Map<targets::Key, Node>>, pub actions: Seq<Action> }
pub open spec fn capture(n: Node) -> Stored {
    Stored { base:n.log.base,base_term:n.log.base_term,prefix:n.prefix,snapshot:n.snapshot,term: n.election.term, vote: n.election.vote, entries: n.log.entries, commit: n.log.commit }
}
pub open spec fn tip(entries: Seq<replication::Entry>) -> u64 {
    if entries.len() == 0 { 0 } else { entries.last().term }
}
pub open spec fn sync(e: election::Image, l: replication::Image) -> election::Image {
    election::Image { last_index: (l.base+l.entries.len()) as u64, last_term: if l.entries.len()==0 {l.base_term}else{tip(l.entries)}, ..e }
}
pub open spec fn initial(n: Node, key: targets::Key) -> bool {
    election::initial(n.election) && n.election.me == key.node
    && n.prefix.len()==0 && n.snapshot.is_none() && n.log.base==0 && n.log.base_term==0 && n.log.entries.len() == 0 && n.log.commit == 0 && n.log.applied == 0
    && n.disk is Missing && n.pending.is_none() && n.alive
}
pub open spec fn stored_valid(d: Stored, members: usize) -> bool {
    d.base+d.entries.len() <= u64::MAX && d.entries.len() <= usize::MAX && d.base<=d.commit<=d.base+d.entries.len()
    && d.prefix.len()==d.base && d.base_term==tip(d.prefix)
    && (d.vote.is_some() ==> d.vote.unwrap() < members)
}
pub open spec fn local(n: Node) -> bool {
    election::well_formed(n.election) && replication::valid_image(n.log)
    && n.prefix.len()==n.log.base && n.log.base_term==tip(n.prefix)
    && n.election.last_index == n.log.base+n.log.entries.len()
    && n.election.last_term == if n.log.entries.len()==0 {n.log.base_term}else{tip(n.log.entries)}
    && (match n.disk { Disk::Complete(d) => stored_valid(d, n.election.members), _ => true })
    && (n.pending.is_some() ==> stored_valid(n.pending.unwrap(), n.election.members))
}
pub open spec fn preserved_host(a: Node, b: Node) -> bool {
    b.disk == a.disk && b.pending == a.pending && b.alive == a.alive
    && b.prefix==a.prefix && b.snapshot==a.snapshot
}
pub open spec fn core_step(a: Node, input: Input, b: Node) -> bool {
    preserved_host(a,b) && a.alive && a.pending.is_none()
    && match input {
        Input::Election(i) => !(i is Crash) && !(i is Restart)
            && b.election == election::next(a.election,i) && b.log == a.log,
        Input::Propose(entry) => a.election.role == election::Role::Leader
            && entry.term == a.election.term && a.log.base+a.log.entries.len() < u64::MAX && a.log.entries.len() < usize::MAX
            && b.log == replication::Image { entries: a.log.entries.push(entry), ..a.log }
            && b.election == sync(a.election,b.log),
        Input::Append { index,entry } => a.election.role==election::Role::Follower && b.log == replication::appended(a.log,index,entry)
            && b.election == sync(a.election,b.log),
        Input::FollowerCommit { advertised,matched } => {
            let bound = replication::min64(replication::min64(advertised,matched),(a.log.base+a.log.entries.len()) as u64);
            b.log == replication::Image { commit: if bound > a.log.commit { bound } else { a.log.commit }, ..a.log }
            && b.election == a.election
        },
        Input::LeaderCommit { matched } => b.election == a.election && b.log.entries == a.log.entries
            && b.log.applied == a.log.applied && a.log.commit <= b.log.commit <= a.log.base+a.log.entries.len()
            && b.log.base==a.log.base && b.log.base_term==a.log.base_term
            && (b.log.commit > a.log.commit ==> matched.len() > 0 && a.election.me < matched.len()
                && a.log.entries[b.log.commit as int-a.log.base as int-1].term == a.election.term
                && replication::tally(matched,matched.len(),b.log.commit,a.election.me) > matched.len()/2),
        Input::Apply { delivered: _ } => b.election == a.election && b.log == replication::Image {
            applied: if a.log.applied < a.log.commit { (a.log.applied+1) as u64 } else { a.log.applied }, ..a.log },
        _ => false,
    }
}
pub open spec fn step(a: Node, input: Input, b: Node) -> bool {
    match input {
        Input::Compact{index,bytes}=>a.alive && a.pending.is_none() && a.log.base<index<=a.log.applied
            && b==Node{prefix:a.prefix+a.log.entries.take(index as int-a.log.base as int),
                snapshot:Some(Snapshot{index,term:a.log.entries[index as int-a.log.base as int-1].term,bytes}),
                log:replication::Image{base:index,base_term:a.log.entries[index as int-a.log.base as int-1].term,
                    entries:a.log.entries.skip(index as int-a.log.base as int),..a.log},..a},
        Input::Install{index,term,bytes,prefix}=>a.election.role==election::Role::Follower && a.alive && a.pending.is_none() && index>a.log.commit
            && prefix.len()==index && tip(prefix)==term
            && b==Node{prefix,snapshot:Some(Snapshot{index,term,bytes}),
                election:sync(a.election,b.log),log:replication::Image{base:index,base_term:term,
                    commit:index,applied:index,entries:if index<=a.log.base+a.log.entries.len()
                        && a.log.entries[index as int-a.log.base as int-1].term==term {
                        a.log.entries.skip(index as int-a.log.base as int)}else{Seq::empty()}},..a},
        Input::DeliverSnapshot=>a.alive && a.snapshot.is_some() && b==a,
        Input::BeginWrite => a.alive && a.pending.is_none() && b == Node {
            disk: Disk::Incomplete, pending: Some(capture(a)), ..a },
        Input::CompleteWrite => a.alive && a.pending.is_some() && b == Node {
            disk: Disk::Complete(a.pending.unwrap()), pending: None, ..a },
        Input::Crash => b == Node { alive: false, pending: None,
            election: election::Image { role: election::Role::Follower, ..a.election }, ..a },
        Input::Restart => !a.alive && match a.disk {
            Disk::Complete(d) => {
                let l = replication::Image { base:d.base,base_term:d.base_term,entries: d.entries, commit: d.commit, applied: d.base };
                b == Node { alive: true, pending: None, log: l,prefix:d.prefix,snapshot:d.snapshot,election: sync(election::Image {
                    term: d.term, role: election::Role::Follower, vote: d.vote,
                    grants: Seq::new(a.election.members as nat, |j:int| false), count: 0, ..a.election },l), ..a }
            },
            // Missing initial storage has never authorized protocol outputs.
            // An incomplete existing image fails construction and emits nothing.
            _ => b == a,
        },
        _ => core_step(a,input,b),
    }
}
pub open spec fn execution(run: Execution) -> bool {
    run.states.len() == run.actions.len()+1
    && (forall |k:targets::Key| run.states[0].contains_key(k) ==> initial(run.states[0][k],k))
    && forall |t:int| #![trigger run.actions[t]] 0 <= t < run.actions.len() ==> {
        let k=run.actions[t].who;
        run.states[t].contains_key(k) && run.states[t+1].contains_key(k)
        && run.states[t+1].dom() == run.states[t].dom()
        && step(run.states[t][k],run.actions[t].input,run.states[t+1][k])
        && forall |other:targets::Key| other != k && run.states[t].contains_key(other)
            ==> run.states[t+1][other] == run.states[t][other]
    }
}
pub proof fn step_preserves(a: Node, input: Input, b: Node)
    requires local(a), step(a,input,b),
    ensures local(b),
{
    match input {
        Input::Election(i) => { election::next_preserves(a.election,i); },
        Input::Append {index,entry} => {},
        _ => {},
    }
}
// This lemma deliberately excludes restart: a durable-output invariant is
// still required to connect recovered votes to votes observed before a crash.
pub proof fn step_vote_stable(a:Node,input:Input,b:Node)
    requires local(a), step(a,input,b), !(input is Restart),
        a.election.vote.is_some(), b.election.term==a.election.term,
    ensures b.election.vote==a.election.vote,
{
    match input {
        Input::Election(i)=>{election::vote_stable_in_term(a.election,i);},
        _=>{},
    }
}
pub proof fn state_induction(run: Execution,t:int)
    requires execution(run),0<=t<run.states.len(),
    ensures forall |k:targets::Key| run.states[t].contains_key(k) ==> local(run.states[t][k]),
    decreases t,
{
    if t==0 {
        assert forall |k:targets::Key| run.states[0].contains_key(k) implies local(run.states[0][k]) by {
            assert(initial(run.states[0][k],k));
        }
    } else {
        state_induction(run,t-1);
        let action=run.actions[t-1];
        assert(run.states[t].dom() == run.states[t-1].dom());
        assert(step(run.states[t-1][action.who], action.input, run.states[t][action.who]));
        assert forall |k:targets::Key| run.states[t].contains_key(k) implies local(run.states[t][k]) by {
            assert(run.states[t-1].contains_key(k));
            if k==action.who { step_preserves(run.states[t-1][k],action.input,run.states[t][k]); }
            else { assert(run.states[t][k]==run.states[t-1][k]); }
        }
    }
}
pub open spec fn logical(entries:Seq<replication::Entry>)->Seq<targets::Entry>{
    Seq::new(entries.len(),|i:int|targets::Entry{term:entries[i].term as nat,command:entries[i].data@})
}
pub open spec fn view(n:Node) -> targets::View {
    targets::View { term:n.election.term as nat, leader:n.alive && n.election.role==election::Role::Leader,
        log:logical(n.prefix+n.log.entries),snapshot:match n.snapshot {
            None=>None,Some(s)=>Some(targets::Snapshot{through:s.index as nat,last_term:s.term as nat,bytes:s.bytes}) } }
}
pub open spec fn event(k:targets::Key,a:Node,b:Node,input:Input) -> targets::Event {
    match input {
        Input::Compact{..}=>targets::Event::CreateSnapshot{who:k,prefix:logical(b.prefix),snapshot:targets::Snapshot{
            through:b.log.base as nat,last_term:b.log.base_term as nat,bytes:b.snapshot.unwrap().bytes}},
        Input::DeliverSnapshot=>targets::Event::ApplySnapshot{who:k,prefix:logical(a.prefix),snapshot:targets::Snapshot{
            through:a.log.base as nat,last_term:a.log.base_term as nat,bytes:a.snapshot.unwrap().bytes}},
        Input::CompleteWrite | Input::Restart=>if b.snapshot.is_some(){
            let snap=targets::Snapshot{through:b.log.base as nat,last_term:b.log.base_term as nat,bytes:b.snapshot.unwrap().bytes};
            if input is Restart {targets::Event::RestoreSnapshot{who:k,prefix:logical(b.prefix),snapshot:snap}}
            else{targets::Event::PersistSnapshot{who:k,prefix:logical(b.prefix),snapshot:snap}}
        }else{targets::Event::Silent},
        Input::Apply {delivered} => if delivered && b.log.applied>a.log.applied {
            targets::Event::Apply {who:k,index:b.log.applied as nat,
                entry:targets::Entry {term:a.log.entries[a.log.applied as int-a.log.base as int].term as nat,
                    command:a.log.entries[a.log.applied as int-a.log.base as int].data@}}
        } else {targets::Event::Silent},
        _ => if b.log.commit>a.log.commit {
            targets::Event::Commit {who:k,term:b.election.term as nat,through:b.log.commit as nat}
        } else if a.election.role!=election::Role::Leader && b.election.role==election::Role::Leader && b.alive {
            targets::Event::Elect {who:k,term:b.election.term as nat}
        } else {targets::Event::Silent},
    }
}
pub open spec fn observe(run:Execution) -> targets::History {
    targets::History {states:Seq::new(run.states.len(),|t:int| Map::new(
        |k:targets::Key| run.states[t].contains_key(k),|k:targets::Key|view(run.states[t][k]))),
        events:Seq::new(run.actions.len(),|t:int| {
            let k=run.actions[t].who;
            event(k,run.states[t][k],run.states[t+1][k],run.actions[t].input)
        })}
}
pub proof fn observation_shape(run:Execution)
    requires execution(run), ensures targets::shape(observe(run)),
{
    let h=observe(run);
    assert forall |t:int| 0<=t<h.events.len() implies match #[trigger] h.events[t] {
        targets::Event::Elect{who,term} => h.states[t+1].contains_key(who) && h.states[t+1][who].leader && h.states[t+1][who].term==term,
        targets::Event::Commit{who,term,through} => h.states[t+1].contains_key(who) && h.states[t+1][who].term==term && 1<=through<=h.states[t+1][who].log.len(),
        targets::Event::Apply{who:_,index,entry:_} => index>=1,
        _=>true,
    } by {
        state_induction(run,t+1);
        let k=run.actions[t].who;
        assert(step(run.states[t][k],run.actions[t].input,run.states[t+1][k]));
        assert(run.states[t+1].contains_key(k));
    }
}
pub proof fn execution_witness() -> (run:Execution)
    ensures execution(run),targets::shape(observe(run)),
        exists |i:int| 0<=i<observe(run).events.len() && !(observe(run).events[i] is Silent),
{
    let k=targets::Key{group:0,node:0};
    let e=election::Image{term:0,role:election::Role::Follower,vote:None,me:0,members:1,
        last_term:0,last_index:0,grants:seq![false],count:0};
    let zero=Node{prefix:Seq::empty(),snapshot:None,election:e,log:replication::Image{base:0,base_term:0,entries:Seq::empty(),commit:0,applied:0},
        disk:Disk::Missing,pending:None,alive:true};
    let campaign=Node{election:election::next(e,election::Input::Timeout),..zero};
    let reply=election::Input::Reply{term:1,peer:0,granted:true};
    let leader=Node{election:election::next(campaign.election,reply),..campaign};
    let a=Map::empty().insert(k,zero);
    let b=Map::empty().insert(k,campaign);
    let c=Map::empty().insert(k,leader);
    let run=Execution{states:seq![a,b,c],actions:seq![
        Action{who:k,input:Input::Election(election::Input::Timeout)},
        Action{who:k,input:Input::Election(reply)}]};
    assert(e.grants =~= Seq::new(1,|j:int|false));
    assert forall |key:targets::Key| a.contains_key(key) implies initial(a[key],key) by {
        assert(key==k);
    }
    assert forall |t:int| #![trigger run.actions[t]] 0<=t<run.actions.len() implies {
        let key=run.actions[t].who;
        run.states[t].contains_key(key) && run.states[t+1].contains_key(key)
        && run.states[t+1].dom()==run.states[t].dom()
        && step(run.states[t][key],run.actions[t].input,run.states[t+1][key])
        && forall |other:targets::Key| other!=key && run.states[t].contains_key(other)
            ==> run.states[t+1][other]==run.states[t][other]
    } by { assert(t==0 || t==1); }
    assert(execution(run));
    observation_shape(run);
    assert(observe(run).events[1] is Elect);
    run
}

pub proof fn leader_step(a:Node,b:Node,input:Input)
    requires local(a),local(b),step(a,input,b),view(a).leader,view(b).leader,view(a).term==view(b).term,
    ensures targets::prefix(view(a).log,view(b).log),
{
    match input {
        Input::Compact{index,bytes}=>{
            let cut=index as int-a.log.base as int;
            assert(a.log.entries.take(cut)+a.log.entries.skip(cut) =~= a.log.entries);
            assert((a.prefix+a.log.entries.take(cut))+a.log.entries.skip(cut) =~= a.prefix+a.log.entries);
        },
        Input::Propose(entry)=>{
            assert((b.prefix+b.log.entries).take((a.prefix+a.log.entries).len() as int) =~= a.prefix+a.log.entries);
        },
        _=>{},
    }
    assert(view(a).log =~= view(b).log.take(view(a).log.len() as int));
}
pub proof fn prove_leader_append_only(run:Execution)
    requires execution(run),ensures targets::leader_append_only(observe(run)),
{
    let h=observe(run);
    assert forall |t:int,k:targets::Key| #![trigger h.states[t][k]] 0<=t<h.events.len()
        && h.states[t].contains_key(k) && h.states[t+1].contains_key(k)
        && h.states[t][k].leader && h.states[t+1][k].leader && h.states[t][k].term==h.states[t+1][k].term
        implies targets::prefix(h.states[t][k].log,h.states[t+1][k].log) by {
        state_induction(run,t);state_induction(run,t+1);
        let action=run.actions[t];
        assert(step(run.states[t][action.who],action.input,run.states[t+1][action.who]));
        assert(run.states[t].contains_key(k));assert(run.states[t+1].contains_key(k));
        assert(h.states[t][k]==view(run.states[t][k]));assert(h.states[t+1][k]==view(run.states[t+1][k]));
        if action.who==k {leader_step(run.states[t][k],run.states[t+1][k],action.input);}
        else {assert(run.states[t+1][k]==run.states[t][k]);
            assert(h.states[t][k].log.take(h.states[t][k].log.len() as int) =~= h.states[t][k].log);
        }
    }
}

}
