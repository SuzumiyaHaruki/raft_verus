// Included in vote_tests: all original eight receiver cases remain required.
use vote_api::{Campaign, Send};

fn campaign(c: &mut Core) -> Write {
    let before = c.inspect();
    let Campaign::Save(w) = c.start_election() else { panic!("eligible idle node must start") };
    let Action::Send(send) = w.action else { panic!("campaign must prepare Send") };
    assert_eq!(w.record.state,c.inspect().state);
    assert_eq!(w.record.op,before.next);
    assert_eq!(send,Send{config:before.config,request:request(before.state.term+1,before.config.node,
        before.state.log.term,before.state.log.index)});
    assert_eq!(c.inspect().state.voted_for,Some(before.config.node));
    assert_eq!(c.inspect().role,vote_api::Role::Candidate);
    w
}
fn authorize(c: &mut Core,w: Write) {
    assert_eq!(c.complete(Completion{op:w.record.op,success:true}),Some(w.action));
    assert!(matches!(w.action,Action::Send(_)));
    assert_eq!(c.complete(Completion{op:w.record.op,success:true}),None);
}
#[test]
fn mixed_choices_and_shared_state() {
    let mut c=core(3,None,Log{term:2,index:10});
    let a=begin(&mut c,request(3,1,2,10)); assert!(finish(&mut c,a).granted);
    let w=campaign(&mut c); assert_eq!(w.record.state.term,4);
    let pending=c.inspect();
    assert_eq!(c.receive(request(5,2,2,10)),Receive::Reject(Fault::Busy));
    assert_eq!(c.start_election(),Campaign::Reject(Fault::Busy));
    assert_eq!(c.complete(Completion{op:0,success:true}),None); assert_eq!(c.inspect(),pending);
    authorize(&mut c,w);
    let b=begin(&mut c,request(4,2,2,10)); assert!(!finish(&mut c,b).granted);
    let high=begin(&mut c,request(5,2,1,100));
    assert_eq!((c.inspect().state.term,c.inspect().state.voted_for,c.inspect().role),
        (5,None,vote_api::Role::Follower)); assert!(!finish(&mut c,high).granted);
    let w=campaign(&mut c); assert_eq!(w.record.state.term,6); authorize(&mut c,w);
    let high=begin(&mut c,request(7,2,3,1)); assert!(finish(&mut c,high).granted);
    assert_eq!(c.inspect().role,vote_api::Role::Follower);
}
#[test]
fn campaign_guards_completion_and_max() {
    let mut c=core(0,None,Log{term:0,index:0});
    let w=campaign(&mut c); let pending=c.inspect();
    for op in [1,u64::MAX] { for success in [false,true] {
        assert_eq!(c.complete(Completion{op,success}),None); assert_eq!(c.inspect(),pending);
    }}
    assert_eq!(c.complete(Completion{op:w.record.op,success:false}),None);
    let halted=c.inspect(); assert!(halted.halted);
    assert_eq!(c.start_election(),Campaign::Reject(Fault::Halted));
    assert_eq!(c.receive(request(2,1,0,0)),Receive::Reject(Fault::Halted));
    assert_eq!(c.complete(Completion{op:w.record.op,success:true}),None); assert_eq!(c.inspect(),halted);
    let config=Config{node:0,members:3};
    let mut leader=Core::new(config,Initial{state:Persistent{term:1,voted_for:Some(0),log:Log{term:0,index:0}},
        role:vote_api::Role::Leader}).unwrap(); let old=leader.inspect();
    assert_eq!(leader.start_election(),Campaign::Reject(Fault::Leader)); assert_eq!(leader.inspect(),old);
    let mut max=core(u64::MAX,None,Log{term:0,index:0});let old=max.inspect();
    assert_eq!(max.start_election(),Campaign::Reject(Fault::TermExhausted));assert_eq!(max.inspect(),old);
    let mut exhausted=Core::recover(config,Record{op:u64::MAX-1,state:Persistent{term:1,voted_for:None,log:Log{term:0,index:0}}}).unwrap();
    let old=exhausted.inspect();assert_eq!(exhausted.start_election(),Campaign::Reject(Fault::Exhausted));assert_eq!(exhausted.inspect(),old);
    let mut last=Core::recover(config,Record{op:u64::MAX-2,state:Persistent{term:u64::MAX-1,voted_for:None,
        log:Log{term:u64::MAX-1,index:u64::MAX}}}).unwrap();
    let w=campaign(&mut last);assert_eq!((w.record.op,w.record.state.term),(u64::MAX-1,u64::MAX));
    authorize(&mut last,w);assert_eq!(last.inspect().next,u64::MAX);
}
async fn cluster(nonempty: bool) -> (Vec<NodeHandle>,Vec<RaftHandle>,Vec<async_channel::Receiver<Request>>) {
    NetSim::current().update_config(|c| {c.packet_loss_rate=0.;c.send_latency=Duration::from_millis(1)..Duration::from_millis(2);});
    let mut nodes=Vec::new();let mut handles=Vec::new();let mut observations=Vec::new();
    for i in 0..3 {
        let n=Handle::current().create_node().ip(peers()[i].ip()).build();
        if nonempty && i==0 { n.spawn(async {
            let bytes:Vec<u8>=[6u64,3,0,0,2,10].iter().flat_map(|w|w.to_le_bytes()).collect();
            fs::File::create("state").await.unwrap().write_all_at(&bytes,0).await.unwrap();
        }).await.unwrap(); }
        let h=n.spawn(async move {RaftHandle::new(peers(),i).await.0}).await.unwrap();
        let (tx,rx)=async_channel::bounded(32);h.vote_hooks.lock().unwrap().incoming=Some(tx);
        nodes.push(n);handles.push(h);observations.push(rx);
    }
    (nodes,handles,observations)
}
async fn campaign_rpc(node:&NodeHandle,h:&RaftHandle) -> Result<Vec<(u64,std::result::Result<(u64,bool),String>)>> {
    let h=h.clone();bounded(node.spawn(async move {h.start_election().await})).await.unwrap()
}
async fn sent_to_both(seen:&[async_channel::Receiver<Request>],expected:Request) {
    for rx in &seen[1..] {assert_eq!(bounded(rx.recv()).await.unwrap(),expected,"real registered receiver must see exact round");}
}
#[madsim::test]
async fn rpc_campaign_real_peers_and_shared_term() {
    let (nodes,h,seen)=cluster(true).await;
    let replies=campaign_rpc(&nodes[0],&h[0]).await.unwrap();
    assert_eq!(replies.len(),2);assert!(replies.iter().any(|(_,r)|matches!(r,Ok((4,true)))));
    sent_to_both(&seen,request(4,0,2,10)).await;
    file(&nodes[0],[7,4,1,0,2,10],"campaign self vote must precede sends").await;
    assert_eq!(h[0].term(),4);assert!(h[0].is_leader());
    let reply=call(&nodes[1],&h[1].endpoint,request(5,1,1,100)).await.unwrap();
    assert_eq!((reply.term,reply.vote_granted),(5,false));assert_eq!(h[0].term(),5);
    file(&nodes[0],[8,5,0,0,2,10],"shared higher-term rejection persists").await;
    assert!(call(&nodes[1],&h[1].endpoint,request(5,1,3,1)).await.unwrap().vote_granted);
    campaign_rpc(&nodes[0],&h[0]).await.unwrap();sent_to_both(&seen,request(6,0,2,10)).await;
    assert_eq!(h[0].term(),6);assert!(h[0].is_leader());
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn rpc_campaign_delayed_save() {
    let (nodes,h,seen)=cluster(false).await;
    let (entered,waiting)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    h[0].vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:0,after_write:false,entered,release:released});
    let cloned=h[0].clone();let running=nodes[0].spawn(async move {cloned.start_election().await});
    bounded(waiting.recv()).await.unwrap();
    tally(&h[0].inner.lock().unwrap().vote,&[]);
    assert!(nodes[0].spawn(fs::read("state")).await.unwrap().is_err(),"barrier is before actual save");
    for rx in &seen[1..] { assert!(timeout(Duration::from_millis(30),rx.recv()).await.is_err(),"no campaign RPC before save"); }
    release.send(()).await.unwrap();let replies=bounded(running).await.unwrap().unwrap();
    assert!(replies.iter().any(|(_,r)|matches!(r,Ok((1,true)))));
    sent_to_both(&seen,request(1,0,0,0)).await;
    file(&nodes[0],[0,1,1,0,0,0],"authorized campaign must have actual file").await;
    for n in nodes {Handle::current().kill(n.id());}
}
#[madsim::test]
async fn rpc_campaign_failed_save() {
    let (nodes,h,seen)=cluster(false).await;h[0].vote_hooks.lock().unwrap().fail_write=true;
    assert!(campaign_rpc(&nodes[0],&h[0]).await.is_err());
    for rx in &seen[1..] {assert!(timeout(Duration::from_millis(30),rx.recv()).await.is_err(),"failed save cannot send");}
    assert!(h[0].inner.lock().unwrap().vote.inspect().halted);
    tally(&h[0].inner.lock().unwrap().vote,&[]);
    assert!(call(&nodes[1],&h[1].endpoint,request(2,1,0,0)).await.is_err(),"receiver cannot bypass halted campaign");
    assert!(campaign_rpc(&nodes[0],&h[0]).await.is_err());
    for n in nodes {Handle::current().kill(n.id());}
}
async fn self_restart(pending:bool) {
    let (mut nodes,mut handles,seen)=cluster(false).await;
    let node=nodes.remove(0);let h=handles.remove(0);let rt=Handle::current();let id=node.id();
    let (entered,waiting)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    if pending {h.vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:0,after_write:true,entered,release:released});}
    let cloned=h.clone();let mut running=Some(node.spawn(async move {cloned.start_election().await}));
    if pending {
        let w=bounded(waiting.recv()).await.unwrap();assert_eq!(h.inner.lock().unwrap().vote.inspect().pending,Some(w));
        for rx in &seen[1..] {assert!(timeout(Duration::from_millis(30),rx.recv()).await.is_err(),"written pending cannot send");}
    } else {
        bounded(running.take().unwrap()).await.unwrap().unwrap();sent_to_both(&seen,request(1,0,0,0)).await;
    }
    file(&node,[0,1,1,0,0,0],"pre-kill actual self vote").await;
    let old=Arc::downgrade(&h.inner);rt.kill(id);
    if let Some(running)=running {assert!(bounded(running).await.unwrap_err().is_cancelled());}
    drop(h);drop(node);drop(release);
    bounded(async {while old.upgrade().is_some(){task::yield_now().await;}}).await;
    rt.restart(id);let node=rt.get_node(id).unwrap();
    file(&node,[0,1,1,0,0,0],"retained self vote").await;
    let fresh=start(&node,true).await;let v=fresh.inner.lock().unwrap().vote.inspect();
    assert_eq!((v.state.term,v.state.voted_for,v.role,v.next,v.pending),
        (1,Some(0),vote_api::Role::Follower,1,None),"recover self vote and next from file");
    tally(&fresh.inner.lock().unwrap().vote,&[]);
    assert_eq!(fresh.inner.lock().unwrap().vote.collect(context(1,1),delivered(1,true)),Collect::Ignored);
    tally(&fresh.inner.lock().unwrap().vote,&[]);
    for rx in &seen[1..] {assert!(timeout(Duration::from_millis(30),rx.recv()).await.is_err(),"restart must not resend old campaign");}
    assert_eq!(fresh.inner.lock().unwrap().vote.complete(Completion{op:0,success:true}),None);
    assert!(!call(&nodes[0],&handles[0].endpoint,request(1,1,0,0)).await.unwrap().vote_granted);
    let (entered,waiting)=async_channel::bounded(1);let (release,released)=async_channel::bounded(1);
    fresh.vote_hooks.lock().unwrap().barrier=Some(vote_host::Barrier{op:2,after_write:false,entered,release:released});
    let cloned=fresh.clone();let running=node.spawn(async move {cloned.start_election().await});
    let w=bounded(waiting.recv()).await.unwrap();assert_eq!((w.record.op,w.record.state.term),(2,2));
    assert_eq!(fresh.inner.lock().unwrap().vote.complete(Completion{op:0,success:true}),None);
    assert_eq!(fresh.inner.lock().unwrap().vote.inspect().pending,Some(w));
    release.send(()).await.unwrap();bounded(running).await.unwrap().unwrap();
    sent_to_both(&seen,request(2,0,0,0)).await;
    file(&node,[2,2,1,0,0,0],"post-recovery campaign uses next term and fresh ID").await;
    assert_eq!(fresh.vote_count(),2);assert!(fresh.has_vote(0));
    assert!(fresh.is_leader());rt.kill(id);for n in nodes {rt.kill(n.id());}
}
#[madsim::test]
async fn self_restart_confirmed() {self_restart(false).await;}
#[madsim::test]
async fn self_restart_written_pending() {self_restart(true).await;}
#[madsim::test]
async fn single_member_saved_majority_promotes() {
    let n=Handle::current().create_node().ip(peers()[0].ip()).build();
    n.spawn(async {
        let (h,_)=RaftHandle::new(vec![peers()[0]],0).await;
        assert!(h.start_election().await.unwrap().is_empty());assert_eq!(h.term(),1);assert!(h.is_leader());
        assert_eq!(h.inner.lock().unwrap().vote.inspect().role,vote_api::Role::Leader);
        tally(&h.inner.lock().unwrap().vote,&[0]);
    }).await.unwrap();Handle::current().kill(n.id());
}
