// Independent observations; no direct assignment of a candidate's role or votes.
use election_observation::{Change,Election,observe as observe_election};
fn watch(h:&RaftHandle) -> Arc<Mutex<Vec<Change>>> {
    let log=Arc::new(Mutex::new(Vec::new()));h.vote_hooks.lock().unwrap().changes=Some(log.clone());log
}
fn observed(changes:&[Change]) -> Vec<Election> {
    changes.iter().filter_map(|&c|observe_election(c)).collect()
}
fn assert_history(events:&[Election]) {
    for a in events {for b in events {
        if a.term==b.term {assert_eq!(a.node,b.node,"historical same-term election conflict");}
    }}
}
fn assert_exact_history(changes:&[Change],events:&[Election]) {
    assert_eq!(observed(changes),events,"every actual election must remain recorded");
    assert_history(events);
}
fn fixture_view(node:u64,role:vote_api::Role,term:u64) -> vote_api::View {
    vote_api::View{config:Config{node,members:3},state:Persistent{term,voted_for:Some(node),log:Log{term:0,index:0}},
        role,next:1,pending:None,halted:false,round:Some(term),self_saved:true}
}
#[test]
fn election_observation_calibration() {
    use vote_api::Role::*;
    // Synthetic observation inputs only, never injected into an actual Core.
    let a=Change{node:0,before:fixture_view(0,Candidate,1),after:fixture_view(0,Leader,1)};
    let down=Change{node:0,before:a.after,after:fixture_view(0,Follower,2)};
    let b=Change{node:1,before:fixture_view(1,Candidate,1),after:fixture_view(1,Leader,1)};
    let events=vec![Election{node:0,term:1}];
    assert_exact_history(&[a,down],&events);
    assert!(std::panic::catch_unwind(||assert_exact_history(&[a],&[])).is_err());
    assert!(std::panic::catch_unwind(||assert_exact_history(&[a,down],&[])).is_err());
    assert!(std::panic::catch_unwind(||assert_history(&observed(&[a,down,b]))).is_err());
    assert_eq!(observe_election(Change{node:0,before:a.after,after:a.after}),None);
    assert_eq!(observe_election(Change{node:0,before:a.after,after:fixture_view(0,Leader,2)}),Some(Election{node:0,term:2}));
}
#[madsim::test]
async fn promotion_schedule_calibration() {
    // Exercise the same FuturesUnordered/barrier ordering without a protocol.
    // The barrier must block C only; it must not block B's completion feedback.
    let (entered,waiting)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    let (ready,observed)=async_channel::bounded(2);
    let worker=task::spawn(async move {
        let mut pending=FuturesUnordered::new();
        for peer in [1,2] {
            let entered=entered.clone();let released=released.clone();
            pending.push(async move {if peer==2 {entered.send(()).await.unwrap();released.recv().await.unwrap();} peer});
        }
        while let Some(peer)=pending.next().await {ready.send(peer).await.unwrap();}
    });
    bounded(waiting.recv()).await.unwrap();
    assert_eq!(bounded(observed.recv()).await.unwrap(),1);
    assert_quiet(observe(&observed).await,"third future really remains blocked");
    release.send(()).await.unwrap();assert_eq!(bounded(observed.recv()).await.unwrap(),2);
    bounded(worker).await.unwrap();assert!(matches!(observe(&observed).await,Observation::Closed));
    // A premature feedback value must be rejected by the same live observer.
    let (sender,receiver)=async_channel::bounded(1);sender.send(2).await.unwrap();
    let wrong=observe(&receiver).await;
    assert!(std::panic::catch_unwind(||assert_quiet(wrong,"premature third completion")).is_err());
}
// Actual self-save, with authorized outgoing requests left for a chosen schedule.
async fn saved_request(node:&NodeHandle,h:&RaftHandle) -> Send {
    let h=h.clone();bounded(node.spawn(async move {
        let _serial=h.vote_serial.lock().await;
        let Campaign::Save(w)=h.vote_call(|c|c.start_election()) else {panic!("scheduled campaign must start")};
        vote_host::save(w,&vote_host::Hooks::default()).await.unwrap();
        let Some(Action::Send(sent))=h.vote_call(|c|c.complete(Completion{op:w.record.op,success:true})) else {panic!("saved campaign must authorize send")};
        h.vote_call(|c|c.try_become_leader());sent
    })).await.unwrap()
}
async fn real_reply(node:&NodeHandle,h:&RaftHandle,sent:Send,peer:usize) -> (RpcContext,RpcResult) {
    let h=h.clone();bounded(node.spawn(async move {
        let addr=h.inner.lock().unwrap().peers[peer];let r=sent.request;
        let reply=h.endpoint.call_timeout::<RequestVoteArgs>(addr,RequestVoteArgs{term:r.term,candidate_id:r.candidate,
            last_log_term:r.log.term,last_log_index:r.log.index},Duration::from_secs(2)).await.unwrap().unwrap();
        (RpcContext{sent,peer:peer as u64},RpcResult::Reply{term:reply.term,granted:reply.vote_granted})
    })).await.unwrap()
}
async fn consume(node:&NodeHandle,h:&RaftHandle,pair:(RpcContext,RpcResult)) {
    let h=h.clone();bounded(node.spawn(async move{h.deliver_vote_result(pair.0,pair.1).await})).await.unwrap().unwrap();
}
#[madsim::test]
async fn majority_before_last_rpc_and_stable_history() {
    let (nodes,h,_)=cluster(false).await;let log=watch(&h[0]);let collected=collect_observer(&h[0]);
    let (waiting,release)=result_barrier(&h[0],1,2);
    let a=h[0].clone();let run=nodes[0].spawn(async move{a.start_election().await});
    bounded(waiting.recv()).await.unwrap();wait_collected(&collected,1,1).await;
    assert!(h[0].is_leader(),"majority must promote while third real RPC is blocked");
    tally(&h[0].inner.lock().unwrap().vote,&[0,1]);
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    release.send(()).await.unwrap();bounded(run).await.unwrap().unwrap();
    tally(&h[0].inner.lock().unwrap().vote,&[0,1]);
    assert!(!h[0].vote_call(|c|c.try_become_leader()));
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn single_member_waits_for_actual_save() {
    let n=Handle::current().create_node().ip(peers()[0].ip()).build();
    let h=n.spawn(async{RaftHandle::new(vec![peers()[0]],0).await.0}).await.unwrap();let log=watch(&h);
    let (entered,wait)=async_channel::bounded(1);let (go,release)=async_channel::bounded(1);
    h.vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:0,after_write:false,entered,release});
    let a=h.clone();let run=n.spawn(async move{a.start_election().await});bounded(wait.recv()).await.unwrap();
    assert!(!h.is_leader());assert_eq!(h.vote_count(),0);assert_exact_history(&log.lock().unwrap(),&[]);
    go.send(()).await.unwrap();assert!(bounded(run).await.unwrap().unwrap().is_empty());
    assert!(h.is_leader());assert_eq!(h.vote_count(),1);assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    Handle::current().kill(n.id());
}
fn core_group(members:u64) -> Vec<Core> {
    (0..members).map(|node|Core::new(Config{node,members},initial(0,None,Log{term:0,index:0})).unwrap()).collect()
}
fn core_send(c:&mut Core) -> Send {
    let w=campaign(c);let Some(Action::Send(s))=c.complete(Completion{op:w.record.op,success:true}) else {panic!("save must authorize send")};s
}
fn core_reply(c:&mut Core,s:Send) -> RpcResult {
    let w=begin(c,s.request);let r=finish(c,w);RpcResult::Reply{term:r.term,granted:r.granted}
}
#[test]
fn strict_majority_distinct_real_votes() {
    for members in [4,5] {
        let mut nodes=core_group(members);let s=core_send(&mut nodes[0]);let b=core_reply(&mut nodes[1],s);
        let cx=RpcContext{sent:s,peer:1};nodes[0].collect(cx,b);nodes[0].collect(cx,b);
        assert_eq!(nodes[0].vote_count(),2);assert!(!nodes[0].try_become_leader(),"half or duplicate cannot win");
        assert_eq!(nodes[0].inspect().role,vote_api::Role::Candidate);
        let c=core_reply(&mut nodes[2],s);nodes[0].collect(RpcContext{sent:s,peer:2},c);
        assert!(nodes[0].try_become_leader());assert_eq!(nodes[0].inspect().role,vote_api::Role::Leader);
        assert_eq!(nodes[0].vote_count(),3);assert!(!nodes[0].try_become_leader());
    }
}
#[test]
fn promotion_guards_old_round_and_completion() {
    let mut n=core_group(3);let w=campaign(&mut n[0]);
    assert!(!n[0].try_become_leader());n[0].complete(Completion{op:99,success:true});assert!(!n[0].try_become_leader());
    let Some(Action::Send(old))=n[0].complete(Completion{op:w.record.op,success:true}) else {panic!("saved send")};
    let old_reply=core_reply(&mut n[1],old);let new=core_send(&mut n[0]);
    n[0].collect(RpcContext{sent:old,peer:1},old_reply);assert_eq!(n[0].vote_count(),1);assert!(!n[0].try_become_leader());
    let r=core_reply(&mut n[1],new);n[0].collect(RpcContext{sent:new,peer:1},r);
    let saved=begin(&mut n[0],request(2,2,0,0));assert!(!n[0].try_become_leader());
    finish(&mut n[0],saved);assert!(n[0].try_become_leader());
    let higher=begin(&mut n[0],request(3,2,0,0));assert_eq!(n[0].inspect().role,vote_api::Role::Follower);
    n[0].complete(Completion{op:higher.record.op,success:false});assert!(!n[0].try_become_leader());assert!(n[0].inspect().halted);
}
async fn larger_cluster(members:usize) -> (Vec<NodeHandle>,Vec<RaftHandle>) {
    NetSim::current().update_config(|c|{c.packet_loss_rate=0.;c.send_latency=Duration::from_millis(1)..Duration::from_millis(2);});
    let peers:Vec<SocketAddr>=(0..members).map(|i|format!("10.42.0.{}:1",i+1).parse().unwrap()).collect();
    let mut nodes=Vec::new();let mut handles=Vec::new();
    for i in 0..members {
        let n=Handle::current().create_node().ip(peers[i].ip()).build();let addresses=peers.clone();
        let h=n.spawn(async move{RaftHandle::new(addresses,i).await.0}).await.unwrap();
        nodes.push(n);handles.push(h);
    }
    (nodes,handles)
}
#[madsim::test]
async fn competing_candidates_keep_past_election_after_demotion() {
    let (nodes,h)=larger_cluster(5).await;let logs:Vec<_>=h.iter().map(watch).collect();
    let a=saved_request(&nodes[0],&h[0]).await;let b=saved_request(&nodes[1],&h[1]).await;
    // E really grants B, but the successful reply is withheld from B's collect.
    let be=real_reply(&nodes[1],&h[1],b,4).await;assert_eq!(be.1,delivered(1,true));
    for peer in [2,3] {let r=real_reply(&nodes[0],&h[0],a,peer).await;consume(&nodes[0],&h[0],r).await;}
    assert!(h[0].is_leader());assert_eq!(h[0].vote_count(),3);
    let mut denied=Vec::new();
    for peer in [0,2,3] {let r=real_reply(&nodes[1],&h[1],b,peer).await;assert_eq!(r.1,delivered(1,false));denied.push(r);}
    // A genuinely steps down for C's authorized term-2 request.
    let c=saved_request(&nodes[2],&h[2]).await;real_reply(&nodes[2],&h[2],c,0).await;
    assert!(!h[0].is_leader());assert_eq!(h[0].term(),2);
    consume(&nodes[1],&h[1],be).await;consume(&nodes[1],&h[1],be).await;
    for r in denied {consume(&nodes[1],&h[1],r).await;}
    assert_eq!(h[1].vote_count(),2);assert!(!h[1].is_leader());
    let all:Vec<_>=logs.iter().flat_map(|l|observed(&l.lock().unwrap())).collect();
    assert_history(&all);assert_eq!(all,vec![Election{node:0,term:1}],"demotion must retain the earlier winner despite late affirmative old votes");
    for n in nodes {Handle::current().kill(n.id());}
}
#[test]
fn split_real_votes_can_leave_no_leader() {
    let mut n=core_group(4);let a=core_send(&mut n[0]);let b=core_send(&mut n[1]);
    let ca=core_reply(&mut n[2],a);let db=core_reply(&mut n[3],b);
    n[0].collect(RpcContext{sent:a,peer:2},ca);n[1].collect(RpcContext{sent:b,peer:3},db);
    assert_eq!(n[0].vote_count(),2);assert_eq!(n[1].vote_count(),2);
    assert!(!n[0].try_become_leader());assert!(!n[1].try_become_leader());
    assert_eq!(n[0].inspect().role,vote_api::Role::Candidate);assert_eq!(n[1].inspect().role,vote_api::Role::Candidate);
}
#[madsim::test]
async fn elected_node_consumes_higher_old_rpc() {
    let (nodes,h,_)=cluster(false).await;let log=watch(&h[0]);
    let a=saved_request(&nodes[0],&h[0]).await;
    let ab=real_reply(&nodes[0],&h[0],a,1).await;consume(&nodes[0],&h[0],ab).await;assert!(h[0].is_leader());
    saved_request(&nodes[2],&h[2]).await;let c=saved_request(&nodes[2],&h[2]).await;
    let old=real_reply(&nodes[0],&h[0],a,2).await;assert_eq!(old.1,delivered(c.request.term,false));
    consume(&nodes[0],&h[0],old).await;
    assert!(!h[0].is_leader());assert_eq!(h[0].term(),2);tally(&h[0].inner.lock().unwrap().vote,&[]);
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    assert!(!h[0].vote_call(|c|c.try_become_leader()));
    file(&nodes[0],[1,2,0,0,0,0],"higher old reply must really persist after demotion").await;
    for n in nodes {Handle::current().kill(n.id());}
}
