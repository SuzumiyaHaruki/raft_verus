// Fixed assertions use actual queries. Robustness replay is NOT a network theorem.
use vote_api::{Collect,RpcContext,RpcResult};
fn assert_vote_observation(count:u64, actual:&[u64], expected:&[u64]) {
    let distinct:std::collections::BTreeSet<_>=actual.iter().copied().collect();
    assert_eq!(distinct.len(),actual.len(),"duplicate observed member");
    assert_eq!(count,distinct.len() as u64,"count must equal distinct actual members");
    assert_eq!(distinct,expected.iter().copied().collect(),"actual voter membership");
}
fn tally(c:&Core, expected:&[u64]) {
    let members=c.inspect().config.members;
    let actual:Vec<_>=(0..members).filter(|&i|c.has_vote(i)).collect();
    assert_vote_observation(c.vote_count(),&actual,expected);
}
fn assert_original_context(sent:RpcContext, delivered:RpcContext) {
    assert_eq!(sent,delivered,"RPC completion must retain original request and peer");
}
fn context(term:u64,peer:u64) -> RpcContext {
    RpcContext{sent:Send{config:Config{node:0,members:3},request:request(term,0,0,0)},peer}
}
fn delivered(term:u64,granted:bool) -> RpcResult {RpcResult::Reply{term,granted}}
#[test]
fn collection_observation_calibration() {
    assert_vote_observation(2,&[0,1],&[0,1]);
    assert_original_context(context(1,1),context(1,1));
    assert!(std::panic::catch_unwind(||assert_vote_observation(3,&[0,1],&[0,1])).is_err());
    assert!(std::panic::catch_unwind(||assert_vote_observation(3,&[0,1,1],&[0,1])).is_err());
    assert!(std::panic::catch_unwind(||assert_original_context(context(1,1),context(2,1))).is_err());
    assert!(std::panic::catch_unwind(||assert_original_context(context(1,1),context(1,2))).is_err());
}
#[madsim::test]
async fn collection_queue_calibration() {
    let (tx,rx)=async_channel::bounded(4);
    tx.send((context(1,2),delivered(1,true))).await.unwrap();
    tx.send((context(1,1),delivered(1,true))).await.unwrap();
    tx.send((context(1,1),delivered(1,true))).await.unwrap();drop(tx);
    for peer in [2,1,1] {
        let (c,r)=bounded(rx.recv()).await.unwrap();
        assert_original_context(context(1,peer),c);assert_eq!(r,delivered(1,true));
    }
    assert!(matches!(observe(&rx).await,Observation::Closed),"drained closure is cleanup, not quiet");
}
#[test]
fn collection_guards_and_self_save() {
    let mut c=core(0,None,Log{term:0,index:0});
    let w=campaign(&mut c);tally(&c,&[]);assert!(!c.inspect().self_saved);
    assert_eq!(c.collect(context(1,1),delivered(1,true)),Collect::Reject(Fault::Busy));
    assert_eq!(c.complete(Completion{op:1,success:true}),None);tally(&c,&[]);
    authorize(&mut c,w);tally(&c,&[0]);
    assert_eq!(c.collect(context(1,1),delivered(1,true)),Collect::Counted);tally(&c,&[0,1]);
    assert_eq!(c.collect(context(1,1),delivered(1,true)),Collect::Ignored);tally(&c,&[0,1]);
    for (cx,r) in [(context(1,3),delivered(1,true)),(context(1,0),delivered(1,true)),
        (RpcContext{sent:Send{request:request(1,1,0,0),..context(1,2).sent},peer:2},delivered(1,true)),
        (context(1,2),delivered(1,false)),(context(1,2),RpcResult::Error),(context(1,2),delivered(0,true))] {
        assert_ne!(c.collect(cx,r),Collect::Counted);tally(&c,&[0,1]);
    }
    let w=campaign(&mut c);tally(&c,&[]);authorize(&mut c,w);tally(&c,&[0]);
    assert_eq!(c.collect(context(1,1),delivered(1,true)),Collect::Ignored);tally(&c,&[0]);
    assert_eq!(c.collect(context(2,2),delivered(2,true)),Collect::Counted);tally(&c,&[0,2]);
    let w=campaign(&mut c);c.complete(Completion{op:w.record.op,success:false});tally(&c,&[]);
    assert_eq!(c.collect(context(2,1),delivered(9,true)),Collect::Reject(Fault::Halted));
}
#[test]
fn collection_higher_term_and_exhaustion() {
    let mut c=core(0,None,Log{term:0,index:0});let w=campaign(&mut c);authorize(&mut c,w);
    let w=campaign(&mut c);authorize(&mut c,w);
    let Collect::Save(w)=c.collect(context(1,1),delivered(5,false)) else {panic!("higher old RPC must save")};
    assert_eq!((c.inspect().state.term,c.inspect().state.voted_for,c.inspect().role),(5,None,vote_api::Role::Follower));
    assert_eq!((w.request,w.action),(None,Action::Persisted));tally(&c,&[]);
    assert_eq!(c.start_election(),Campaign::Reject(Fault::Busy));
    assert_eq!(c.complete(Completion{op:w.record.op,success:true}),Some(Action::Persisted));tally(&c,&[]);
    let mut last=Core::recover(Config{node:0,members:3},Record{op:u64::MAX-2,state:Persistent{term:1,voted_for:None,log:Log{term:0,index:0}}}).unwrap();
    let w=campaign(&mut last);authorize(&mut last,w);tally(&last,&[0]);
    assert_eq!(last.collect(context(2,1),delivered(3,true)),Collect::Reject(Fault::Exhausted));
    assert!(last.inspect().halted);assert_eq!(last.inspect().next,u64::MAX);tally(&last,&[]);
    assert_eq!(last.inspect().pending,None);
    let fake=Core::new(Config{node:0,members:3},Initial{state:Persistent{term:1,voted_for:Some(0),log:Log{term:0,index:0}},role:vote_api::Role::Candidate}).unwrap();
    tally(&fake,&[]);assert_eq!(fake.inspect().round,None);
}
fn result_barrier(h:&RaftHandle,term:u64,peer:u64) -> (async_channel::Receiver<(RpcContext,RpcResult)>,async_channel::Sender<()>) {
    let (entered,wait)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    h.vote_hooks.lock().unwrap().reply_barrier=Some(vote_host::ReplyBarrier{term,peer,entered,release:released});(wait,release)
}
fn collect_observer(h:&RaftHandle) -> async_channel::Receiver<(RpcContext,RpcResult)> {
    let (tx,rx)=async_channel::bounded(32);h.vote_hooks.lock().unwrap().collected=Some(tx);rx
}
async fn wait_collected(rx:&async_channel::Receiver<(RpcContext,RpcResult)>,term:u64,peer:u64) -> (RpcContext,RpcResult) {
    loop {let item=bounded(rx.recv()).await.unwrap();if item.0.sent.request.term==term && item.0.peer==peer{return item;}}
}
#[madsim::test]
async fn collection_real_peers_and_duplicate() {
    let (nodes,h,_)=cluster(false).await;let observed=collect_observer(&h[0]);
    let (waiting,release)=result_barrier(&h[0],1,2);
    let cloned=h[0].clone();let running=nodes[0].spawn(async move{cloned.start_election().await});
    let (c,result)=bounded(waiting.recv()).await.unwrap();assert_original_context(context(1,2),c);
    let (b_context,b_result)=wait_collected(&observed,1,1).await;tally(&h[0].inner.lock().unwrap().vote,&[0,1]);
    // Replay the REAL B response, with the captured original context. This is API robustness.
    let clone=h[0].clone();
    nodes[0].spawn(async move{clone.deliver_vote_result(b_context,b_result).await}).await.unwrap().unwrap();
    tally(&h[0].inner.lock().unwrap().vote,&[0,1]);assert_eq!(result,delivered(1,true));
    release.send(()).await.unwrap();bounded(running).await.unwrap().unwrap();
    tally(&h[0].inner.lock().unwrap().vote,&[0,1,2]);assert!(!h[0].is_leader());
    for n in nodes{Handle::current().kill(n.id());}
}
#[madsim::test]
async fn collection_delayed_old_and_new_round() {
    let (nodes,h,_)=cluster(false).await;
    let (waiting,release)=result_barrier(&h[0],1,1);let observed=collect_observer(&h[0]);
    let clone=h[0].clone();let old=nodes[0].spawn(async move{clone.start_election().await});
    let (cx,raw)=bounded(waiting.recv()).await.unwrap();assert_original_context(context(1,1),cx);
    wait_collected(&observed,1,2).await;
    let (new_wait,new_release)=result_barrier(&h[0],2,1);
    let clone=h[0].clone();let newer=nodes[0].spawn(async move{clone.start_election().await});
    let (new_context,_)=bounded(new_wait.recv()).await.unwrap();assert_original_context(context(2,1),new_context);
    wait_collected(&observed,2,2).await;tally(&h[0].inner.lock().unwrap().vote,&[0,2]);
    release.send(()).await.unwrap();bounded(old).await.unwrap().unwrap();
    assert_eq!(raw,delivered(1,true));assert_eq!(h[0].term(),2);
    tally(&h[0].inner.lock().unwrap().vote,&[0,2]);
    new_release.send(()).await.unwrap();bounded(newer).await.unwrap().unwrap();
    tally(&h[0].inner.lock().unwrap().vote,&[0,1,2]);
    for n in nodes{Handle::current().kill(n.id());}
}
// Schedule real self-saves and deliver only the last authorized request to one
// peer; all other authorized transmissions are dropped by this test schedule.
async fn authorized_to(node:&NodeHandle,handle:&RaftHandle,peer:usize,rounds:usize) {
    let h=handle.clone();bounded(node.spawn(async move {
        let mut last=None;
        for _ in 0..rounds {
            let _serial=h.vote_serial.lock().await;
            let w=campaign(&mut h.inner.lock().unwrap().vote);
            vote_host::save(w,&vote_host::Hooks::default()).await.unwrap();
            let Some(Action::Send(sent))=h.inner.lock().unwrap().vote.complete(Completion{op:w.record.op,success:true})
                else {panic!("real completed campaign must authorize the setup request")};
            last=Some(sent);
        }
        let r=last.unwrap().request;
        h.endpoint.call_timeout::<RequestVoteArgs>(peers()[peer],RequestVoteArgs{term:r.term,candidate_id:r.candidate,
            last_log_term:r.log.term,last_log_index:r.log.index},Duration::from_secs(2)).await.unwrap().unwrap()
    })).await.unwrap();
}
async fn old_higher_reply_case(fail:bool) {
        let (nodes,h,_)=cluster(false).await;
        // C really authorizes its term-5 request; B really receives and saves it.
        authorized_to(&nodes[2],&h[2],1,5).await;
        // C is also at term 5. Stop it so only B supplies the delayed higher
        // reply; otherwise C would correctly advance A before the old B result.
        Handle::current().kill(nodes[2].id());
        let (waiting,release)=result_barrier(&h[0],1,1);let observed=collect_observer(&h[0]);
        let clone=h[0].clone();let old=nodes[0].spawn(async move{clone.start_election().await});
        let (cx,raw)=bounded(waiting.recv()).await.unwrap();assert_original_context(context(1,1),cx);assert_eq!(raw,delivered(5,false));
        wait_collected(&observed,1,2).await;
        // Direct start + real save + actual complete (no second network round), still the same Core.
        nodes[0].spawn({let h=h[0].clone();async move{
            let _serial=h.vote_serial.lock().await;let w=campaign(&mut h.inner.lock().unwrap().vote);
            vote_host::save(w,&vote_host::Hooks::default()).await.unwrap();authorize(&mut h.inner.lock().unwrap().vote,w);
        }}).await.unwrap();
        assert_eq!(h[0].term(),2);
        let (entered,wait)=async_channel::bounded(1);let (go,proceed)=async_channel::bounded(1);
        h[0].vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:2,after_write:false,entered,release:proceed});
        h[0].vote_hooks.lock().unwrap().fail_write=fail;
        release.send(()).await.unwrap();let w=bounded(wait.recv()).await.unwrap();
        assert_eq!(w.action,Action::Persisted);assert_eq!(h[0].term(),5);tally(&h[0].inner.lock().unwrap().vote,&[]);
        assert_eq!(h[0].inner.lock().unwrap().vote.inspect().role,vote_api::Role::Follower);
        file(&nodes[0],[1,2,1,0,0,0],"higher reply must await its real write").await;
        go.send(()).await.unwrap();let out=bounded(old).await.unwrap();assert_eq!(out.is_err(),fail);
        if fail{assert!(h[0].inner.lock().unwrap().vote.inspect().halted);}else{file(&nodes[0],[2,5,0,0,0,0],"higher old RPC term saved").await;}
        for n in &nodes[..2]{Handle::current().kill(n.id());}
}
#[madsim::test]
async fn collection_old_higher_saved() {old_higher_reply_case(false).await;}
#[madsim::test]
async fn collection_old_higher_failed() {old_higher_reply_case(true).await;}
#[madsim::test]
async fn collection_transport_and_denial() {
    let (nodes,h,_)=cluster(false).await;
    // C really starts/saves before B grants its request. A sees B's real denial.
    authorized_to(&nodes[2],&h[2],1,1).await;
    Handle::current().kill(nodes[2].id());
    let result=campaign_rpc(&nodes[0],&h[0]).await.unwrap();
    assert!(result.iter().any(|(p,r)|*p==1 && matches!(r,Ok((1,false)))));
    assert!(result.iter().any(|(p,r)|*p==2 && r.is_err()),"transport error must be visible");
    tally(&h[0].inner.lock().unwrap().vote,&[0]);
    Handle::current().kill(nodes[0].id());Handle::current().kill(nodes[1].id());
}
