// Fixed multi-node tests. Scheduling uses real authorized Send values and RPCs.
// Shared observers live outside the killed node; no expected state is an input to recovery.
async fn reboot_at(nodes:&mut Vec<NodeHandle>,handles:&mut Vec<RaftHandle>,at:usize,
    log:&Arc<Mutex<Vec<Change>>>,restored:&Arc<Mutex<Vec<vote_api::View>>>) {
    let node=nodes.remove(at);let h=handles.remove(at);let id=node.id();
    let addresses=h.inner.lock().unwrap().peers.clone();let me=h.inner.lock().unwrap().me;
    let retained=bounded(node.spawn(fs::read("state"))).await.unwrap().unwrap();
    let old=Arc::downgrade(&h.inner);Handle::current().kill(id);drop(h);drop(node);
    bounded(async {while old.upgrade().is_some(){task::yield_now().await;}}).await;
    Handle::current().restart(id);let node=Handle::current().get_node(id).unwrap();
    assert_eq!(bounded(node.spawn(fs::read("state"))).await.unwrap().unwrap(),retained,"same node must retain its actual file");
    let hooks=vote_host::Hooks{changes:Some(log.clone()),recovered:Some(restored.clone()),..Default::default()};
    let h=bounded(node.spawn(async move{RaftHandle::new_receiver_observed(addresses,me,true,hooks).await.0})).await.unwrap();
    let v=h.inner.lock().unwrap().vote.inspect();
    assert_eq!(v.role,vote_api::Role::Follower,"restored node must not resurrect leadership");
    assert_eq!((v.pending,v.round,v.self_saved),(None,None,false));tally(&h.inner.lock().unwrap().vote,&[]);
    nodes.insert(at,node);handles.insert(at,h);
}
fn shared_observer(handles:&[RaftHandle])->Arc<Mutex<Vec<Change>>> {
    let log=Arc::new(Mutex::new(Vec::new()));
    for h in handles {h.vote_hooks.lock().unwrap().changes=Some(log.clone());}log
}
#[madsim::test]
async fn cluster_completed_restarts_keep_votes_and_past_leader() {
    let (mut nodes,mut h)=larger_cluster(3).await;let log=shared_observer(&h);let recovered=Arc::new(Mutex::new(Vec::new()));
    let a=saved_request(&nodes[0],&h[0]).await;let c=saved_request(&nodes[2],&h[2]).await;
    let ab=real_reply(&nodes[0],&h[0],a,1).await;consume(&nodes[0],&h[0],ab).await;
    assert!(h[0].is_leader());file(&nodes[1],[0,1,1,0,0,0],"B really saved A").await;
    reboot_at(&mut nodes,&mut h,1,&log,&recovered).await;
    assert_eq!((h[1].term(),h[1].inner.lock().unwrap().vote.inspect().state.voted_for),(1,Some(0)));
    let cb=real_reply(&nodes[2],&h[2],c,1).await;assert_eq!(cb.1,delivered(1,false));consume(&nodes[2],&h[2],cb).await;
    assert!(!h[2].is_leader());file(&nodes[1],[1,1,1,0,0,0],"denial preserves old vote").await;
    reboot_at(&mut nodes,&mut h,1,&log,&recovered).await; // Same voter, second actual instance.
    reboot_at(&mut nodes,&mut h,0,&log,&recovered).await; // Different node, old Leader.
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    let a2=saved_request(&nodes[0],&h[0]).await;assert_eq!(a2.request.term,2);
    let ab2=real_reply(&nodes[0],&h[0],a2,1).await;consume(&nodes[0],&h[0],ab2).await;
    assert!(h[0].is_leader());
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1},Election{node:0,term:2}]);
    let views=recovered.lock().unwrap();assert_eq!(views.len(),3);assert!(views.iter().all(|v|v.state.voted_for==Some(0) && v.state.term==1));
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn cluster_written_unnotified_vote_and_real_retry() {
    let (mut nodes,mut h)=larger_cluster(3).await;let log=shared_observer(&h);let recovered=Arc::new(Mutex::new(Vec::new()));
    let a=saved_request(&nodes[0],&h[0]).await;let c=saved_request(&nodes[2],&h[2]).await;
    let (entered,wait)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    h[1].vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:0,after_write:true,entered,release:released});
    let (done,observed)=async_channel::bounded(1);let ep=h[0].endpoint.clone();let addr=h[0].inner.lock().unwrap().peers[1];
    let running=nodes[0].spawn(async move {
        let r=a.request;let result=ep.call_timeout::<RequestVoteArgs>(addr,RequestVoteArgs{term:r.term,candidate_id:r.candidate,last_log_term:r.log.term,last_log_index:r.log.index},Duration::from_secs(2)).await;
        done.send(result).await.unwrap();
    });
    let w=bounded(wait.recv()).await.unwrap();assert_eq!(h[1].inner.lock().unwrap().vote.inspect().pending,Some(w));
    file(&nodes[1],[0,1,1,0,0,0],"post-write barrier must follow the real complete write").await;
    assert_quiet(observe(&observed).await,"live caller has no premature reply");
    reboot_at(&mut nodes,&mut h,1,&log,&recovered).await;
    assert_eq!(h[1].inner.lock().unwrap().vote.inspect().state.voted_for,Some(0));
    assert_quiet(observe(&observed).await,"restart must not synthesize the old uncompleted reply");
    assert_eq!(h[0].vote_count(),1);assert!(!h[0].is_leader());
    let denied=real_reply(&nodes[2],&h[2],c,1).await;assert_eq!(denied.1,delivered(1,false));consume(&nodes[2],&h[2],denied).await;
    let retry=real_reply(&nodes[0],&h[0],a,1).await;assert_eq!(retry.1,delivered(1,true));consume(&nodes[0],&h[0],retry).await;
    assert!(h[0].is_leader());assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    assert_quiet(observe(&observed).await,"a real retry is distinct from the cancelled old server work");
    running.abort();assert!(bounded(running).await.unwrap_err().is_cancelled());
    assert!(matches!(observe(&observed).await,Observation::Closed));drop(release);
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn cluster_reply_survives_voter_restart_but_not_caller_restart() {
    let (mut nodes,mut h)=larger_cluster(3).await;let log=shared_observer(&h);let recovered=Arc::new(Mutex::new(Vec::new()));
    let a=saved_request(&nodes[0],&h[0]).await;
    let actual=real_reply(&nodes[0],&h[0],a,1).await;assert_eq!(actual.1,delivered(1,true));
    reboot_at(&mut nodes,&mut h,1,&log,&recovered).await;
    consume(&nodes[0],&h[0],actual).await;consume(&nodes[0],&h[0],actual).await;
    assert!(h[0].is_leader());assert_eq!(h[0].vote_count(),2);
    reboot_at(&mut nodes,&mut h,0,&log,&recovered).await;
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    // Now exercise actual driver futures: delay a real term-2 reply, kill its caller.
    let (entered,wait)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    h[0].vote_hooks.lock().unwrap().reply_barrier=Some(vote_host::ReplyBarrier{peer:1,term:2,entered,release:released});
    let old=h[0].clone();let running=nodes[0].spawn(async move{old.start_election().await});
    let (context,reply)=bounded(wait.recv()).await.unwrap();assert_eq!(context.sent.request.term,2);assert_eq!(reply,delivered(2,true));
    reboot_at(&mut nodes,&mut h,0,&log,&recovered).await;
    assert!(bounded(running).await.unwrap_err().is_cancelled(),"killed caller future cannot continue on new Core");
    assert!(release.send(()).await.is_err(),"old reply barrier has no surviving callback");
    tally(&h[0].inner.lock().unwrap().vote,&[]);assert_eq!(h[0].term(),2);assert!(!h[0].is_leader());
    // An old completion also cannot consume a newly accepted operation.
    let (entered,wait)=async_channel::bounded(1);let (go,released)=async_channel::bounded(1);
    h[0].vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:2,after_write:true,entered,release:released});
    let new=h[0].clone();let running=nodes[0].spawn(async move{new.start_election().await});let w=bounded(wait.recv()).await.unwrap();
    let before=h[0].inner.lock().unwrap().vote.inspect();
    assert_eq!(h[0].vote_call(|c|c.complete(Completion{op:1,success:true})),None);
    assert_eq!(h[0].inner.lock().unwrap().vote.inspect(),before);assert_eq!(w.record.op,2);
    go.send(()).await.unwrap();bounded(running).await.unwrap().unwrap();assert!(h[0].is_leader());assert_eq!(h[0].term(),3);
    assert_history(&observed(&log.lock().unwrap()));
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn cluster_persisted_higher_reply_restart() {
    let (mut nodes,mut h)=larger_cluster(3).await;let log=shared_observer(&h);let recovered=Arc::new(Mutex::new(Vec::new()));
    let a=saved_request(&nodes[0],&h[0]).await;let ab=real_reply(&nodes[0],&h[0],a,1).await;consume(&nodes[0],&h[0],ab).await;
    saved_request(&nodes[2],&h[2]).await;saved_request(&nodes[2],&h[2]).await;
    let higher=real_reply(&nodes[0],&h[0],a,2).await;assert_eq!(higher.1,delivered(2,false));consume(&nodes[0],&h[0],higher).await;
    file(&nodes[0],[1,2,0,0,0,0],"Persisted completion must save the higher reply term").await;
    reboot_at(&mut nodes,&mut h,0,&log,&recovered).await;
    let v=h[0].inner.lock().unwrap().vote.inspect();assert_eq!((v.state.term,v.state.voted_for,v.next),(2,None,2));
    assert_exact_history(&log.lock().unwrap(),&[Election{node:0,term:1}]);
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn restart_file_window_calibration() {
    // Protocol-free positive/negative control for the byte oracle and post-write barrier.
    let n=Handle::current().create_node().build();let record=Record{op:3,state:Persistent{term:7,voted_for:Some(2),log:Log{term:4,index:5}}};
    let w=Write{record,request:None,action:Action::Persisted};
    let (entered,wait)=async_channel::bounded(1);let (go,release)=async_channel::bounded(1);
    let h=vote_host::Hooks{barrier:Some(vote_host::Barrier{op:3,after_write:true,entered,release}),..Default::default()};
    let run=n.spawn(async move{vote_host::save(w,&h).await});let seen=bounded(wait.recv()).await.unwrap();assert_eq!(seen,w);
    file(&n,[3,7,1,2,4,5],"real successful write precedes the barrier").await;
    let bytes=bounded(n.spawn(fs::read("state"))).await.unwrap().unwrap();
    assert_ne!(bytes,vec![0;48],"fresh/default memory cannot stand in for the retained record");
    go.send(()).await.unwrap();bounded(run).await.unwrap().unwrap();Handle::current().kill(n.id());
}
