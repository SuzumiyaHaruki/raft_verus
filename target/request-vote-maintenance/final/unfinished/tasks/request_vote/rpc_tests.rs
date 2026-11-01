//! Protected tests: drive the original registered MadRaft RPC, never substitute a voter.
use super::*;
use vote_api::{Completion, Config, Fault, Initial, Log, Persistent, Receive, Record, Request, Write};
use vote_core::Core;
use madsim::{runtime::{Handle, NodeHandle}, net::{Endpoint, NetSim}, time::{timeout, Duration}};
use std::future::Future;

fn initial(term: u64, vote: Option<u64>, log: Log) -> Initial {
    Initial { state:Persistent{term,voted_for:vote,log},role:vote_api::Role::Follower }
}
fn core(term: u64, vote: Option<u64>, log: Log) -> Core {
    Core::new(Config{node:0,members:3},initial(term,vote,log)).unwrap()
}
fn request(term: u64, candidate: u64, log_term: u64, index: u64) -> Request {
    Request{term,candidate,log:Log{term:log_term,index}}
}
fn begin(c: &mut Core, r: Request) -> Write {
    let Receive::Save(w) = c.receive(r) else { panic!("valid idle request must produce decision") };
    assert_eq!(w.request,r);
    assert_eq!(w.record.state,c.inspect().state);
    w
}
fn finish(c: &mut Core, w: Write) -> vote_api::Reply {
    let reply = c.complete(Completion{op:w.record.op,success:true}).expect("matching save must reply");
    assert_eq!(reply,w.reply);
    reply
}
#[test]
fn rules_and_nonempty() {
    let mut c = core(3,None,Log{term:2,index:10});
    let w = begin(&mut c,request(3,1,2,10));
    assert!(finish(&mut c,w).granted,"eligible request must grant");
    for (who,grant) in [(2,false),(1,true)] {
        let w = begin(&mut c,request(3,who,2,10));
        assert_eq!(finish(&mut c,w).granted,grant,"same-term vote cannot change");
    }
    let w = begin(&mut c,request(2,2,2,100));
    assert_eq!((w.reply.term,w.reply.granted),(3,false));
    finish(&mut c,w);
    let w = begin(&mut c,request(4,2,1,100));
    assert_eq!((w.reply.term,w.reply.granted),(4,false));
    assert_eq!(c.inspect().state.voted_for,None);
    assert_eq!(c.inspect().role,vote_api::Role::Follower);
    finish(&mut c,w);
    let w = begin(&mut c,request(4,2,3,1));
    assert!(finish(&mut c,w).granted,"term takes priority over length");
    let mut leader = Core::new(Config{node:0,members:3},Initial{
        state:Persistent{term:3,voted_for:Some(0),log:Log{term:2,index:10}},role:vote_api::Role::Leader}).unwrap();
    let w = begin(&mut leader,request(4,1,1,100));
    assert!(!w.reply.granted);
    assert_eq!(leader.inspect().role,vote_api::Role::Follower);
    assert_eq!(leader.inspect().state.term,4);
}
#[test]
fn invalid_busy_failure_completion_and_bounds() {
    let mut c = core(1,None,Log{term:0,index:0});
    for r in [request(0,1,0,0),request(1,3,0,0),request(1,1,2,1),request(1,1,0,1)] {
        let old = c.inspect();
        assert_eq!(c.receive(r),Receive::Reject(Fault::Invalid));
        assert_eq!(c.inspect(),old);
    }
    let w = begin(&mut c,request(1,1,0,0));
    let pending = c.inspect();
    assert_eq!(c.receive(request(2,2,0,0)),Receive::Reject(Fault::Busy));
    for op in [1,u64::MAX] { for success in [true,false] {
        assert_eq!(c.complete(Completion{op,success}),None);
        assert_eq!(c.inspect(),pending);
    }}
    finish(&mut c,w);
    assert_eq!(c.complete(Completion{op:0,success:true}),None);
    let failed = begin(&mut c,request(2,2,0,0));
    assert_eq!(c.complete(Completion{op:failed.record.op,success:false}),None);
    assert_eq!(c.receive(request(3,1,0,0)),Receive::Reject(Fault::Halted));
    assert_eq!(c.complete(Completion{op:failed.record.op,success:true}),None);
    let state = Persistent{term:u64::MAX,voted_for:Some(1),log:Log{term:u64::MAX,index:u64::MAX}};
    let config = Config{node:0,members:3};
    assert!(Core::recover(config,Record{op:u64::MAX,state}).is_none());
    let mut last = Core::recover(config,Record{op:u64::MAX-1,state}).unwrap();
    assert_eq!(last.inspect().next,u64::MAX);
    assert_eq!(last.receive(request(u64::MAX,1,u64::MAX,u64::MAX)),Receive::Reject(Fault::Exhausted));
    assert!(Core::new(Config{node:3,members:3},initial(1,None,Log{term:0,index:0})).is_none());
    assert!(Core::new(config,initial(1,Some(3),Log{term:0,index:0})).is_none());
    assert!(Core::new(config,initial(1,None,Log{term:2,index:1})).is_none());
}
async fn bounded<T>(f: impl Future<Output=T>) -> T {
    timeout(Duration::from_secs(3),f).await.expect("controlled step watchdog")
}
fn peers() -> Vec<SocketAddr> { ["10.0.0.1:9000","10.0.0.2:9000","10.0.0.3:9000"].iter().map(|a|a.parse().unwrap()).collect() }
async fn start(node: &NodeHandle, recovery: bool) -> RaftHandle {
    bounded(node.spawn(async move {
        if recovery { RaftHandle::new_receiver(peers(),0,true).await.0 }
        else { RaftHandle::new(peers(),0).await.0 }
    })).await.unwrap()
}
async fn call(node: &NodeHandle, endpoint: &Endpoint, r: Request) -> std::result::Result<RequestVoteReply,String> {
    let ep = endpoint.clone();
    bounded(node.spawn(async move {
        ep.call_timeout::<RequestVoteArgs>(peers()[0],RequestVoteArgs{term:r.term,candidate_id:r.candidate,
            last_log_term:r.log.term,last_log_index:r.log.index},Duration::from_secs(2)).await.unwrap()
    })).await.unwrap()
}
async fn network() -> (NodeHandle,NodeHandle,Endpoint,RaftHandle) {
    NetSim::current().update_config(|c| {c.packet_loss_rate=0.; c.send_latency=Duration::from_millis(1)..Duration::from_millis(2);});
    let rt = Handle::current();
    let server = rt.create_node().ip(peers()[0].ip()).build();
    let client = rt.create_node().ip(peers()[1].ip()).build();
    let ep = client.spawn(Endpoint::bind(peers()[1])).await.unwrap().unwrap();
    let handle = start(&server,false).await;
    (server,client,ep,handle)
}
async fn second_client() -> (NodeHandle,Endpoint) {
    let node=Handle::current().create_node().ip(peers()[2].ip()).build();
    let ep=node.spawn(Endpoint::bind(peers()[2])).await.unwrap().unwrap();
    (node,ep)
}
// Independent byte oracle: it does not call the adapter's encoder or decoder.
async fn file(node: &NodeHandle, expected: [u64;6], label: &str) {
    let actual = bounded(node.spawn(fs::read("state"))).await.unwrap()
        .unwrap_or_else(|e| panic!("{}: {}",label,e));
    let expected: Vec<u8> = expected.iter().flat_map(|w|w.to_le_bytes()).collect();
    assert_eq!(actual,expected,"{label}");
}
#[madsim::test]
async fn rpc_rules_and_file() {
    let (server,client,ep,_h) = network().await;
    let (other,other_ep)=second_client().await;
    assert!(call(&client,&ep,request(1,1,0,0)).await.unwrap().vote_granted,"RPC eligible grant");
    file(&server,[0,1,1,1,0,0],"authorized grant must be in real file").await;
    assert!(!call(&other,&other_ep,request(1,2,0,0)).await.unwrap().vote_granted);
    assert!(call(&client,&ep,request(1,1,0,0)).await.unwrap().vote_granted);
    assert!(call(&client,&ep,request(1,3,0,0)).await.is_err());
    Handle::current().kill(server.id());
}
#[madsim::test]
async fn rpc_nonempty_log_and_denial_persist() {
    let rt=Handle::current();
    let server=rt.create_node().ip(peers()[0].ip()).build();
    // Explicit legal log-summary fixture, not an AppendEntries implementation.
    server.spawn(async {
        let words=[6u64,3,1,1,2,10];
        let bytes:Vec<u8>=words.iter().flat_map(|w|w.to_le_bytes()).collect();
        fs::File::create("state").await.unwrap().write_all_at(&bytes,0).await.unwrap();
    }).await.unwrap();
    let _h=start(&server,true).await;
    let (client,ep)=second_client().await;
    let denied=call(&client,&ep,request(4,2,1,100)).await.unwrap();
    assert_eq!((denied.term,denied.vote_granted),(4,false));
    file(&server,[7,4,0,0,2,10],"higher-term denial must persist term and cleared vote").await;
    assert!(call(&client,&ep,request(4,2,3,1)).await.unwrap().vote_granted);
    rt.kill(server.id());
}
#[madsim::test]
async fn rpc_delayed_and_failed_save() {
    let (server,client,ep,h)=network().await;
    let (other,other_ep)=second_client().await;
    let (entered,waiting)=async_channel::bounded(1);
    let (release,released)=async_channel::bounded(1);
    *h.vote_hooks.lock().unwrap()=vote_host::Hooks{barrier:Some(vote_host::Barrier{
        op:0,after_write:false,entered,release:released}),fail_write:false};
    let (done,observed)=async_channel::bounded(1);
    let c=client.clone(); let e=ep.clone();
    let running=task::spawn(async move {done.send(call(&c,&e,request(1,1,0,0)).await).await.unwrap();});
    bounded(waiting.recv()).await.unwrap();
    assert!(timeout(Duration::from_millis(30),observed.recv()).await.is_err(),"no reply before actual write");
    assert!(server.spawn(fs::read("state")).await.unwrap().is_err());
    release.send(()).await.unwrap();
    assert!(bounded(observed.recv()).await.unwrap().unwrap().vote_granted);
    running.await.unwrap();
    *h.vote_hooks.lock().unwrap()=vote_host::Hooks{barrier:None,fail_write:true};
    assert!(call(&other,&other_ep,request(2,2,0,0)).await.is_err(),"failed save must not authorize vote");
    assert!(h.inner.lock().unwrap().vote.inspect().halted);
    Handle::current().kill(server.id());
}
async fn restart_case(uncompleted: bool) {
    let (server,client,ep,h)=network().await;
    let (other,other_ep)=second_client().await;
    let rt=Handle::current(); let id=server.id();
    let (entered,waiting)=async_channel::bounded(1);
    let (release,released)=async_channel::bounded(1);
    if uncompleted {
        *h.vote_hooks.lock().unwrap()=vote_host::Hooks{barrier:Some(vote_host::Barrier{
            op:0,after_write:true,entered,release:released}),fail_write:false};
    }
    let (done,observed)=async_channel::bounded(1);
    let c=client.clone(); let e=ep.clone();
    let running=task::spawn(async move {done.send(call(&c,&e,request(1,1,0,0)).await).await.unwrap();});
    if uncompleted {
        let w=bounded(waiting.recv()).await.unwrap();
        assert_eq!(h.inner.lock().unwrap().vote.inspect().pending,Some(w));
        assert!(timeout(Duration::from_millis(30),observed.recv()).await.is_err(),"no reply at post-write barrier");
    } else {
        assert!(bounded(observed.recv()).await.unwrap().unwrap().vote_granted);
        running.await.unwrap();
    }
    file(&server,[0,1,1,1,0,0],"pre-kill actual saved vote").await;
    // No release and no new accepted operation. Discard ALL old core handles.
    rt.kill(id); assert!(rt.is_exit(id));
    drop(h); drop(server); drop(release);
    rt.restart(id);
    let restarted=rt.get_node(id).unwrap();
    file(&restarted,[0,1,1,1,0,0],"same-node file retention").await;
    let fresh=start(&restarted,true).await;
    let v=fresh.inner.lock().unwrap().vote.inspect();
    assert_eq!((v.state.term,v.state.voted_for,v.next,v.pending),(1,Some(1),1,None),"restore vote from file");
    if uncompleted {
        assert!(timeout(Duration::from_millis(30),observed.recv()).await.is_err(),"restart must not synthesize old reply");
    }
    assert!(!call(&other,&other_ep,request(1,2,0,0)).await.unwrap().vote_granted,"recovery must not forget vote");
    assert!(call(&client,&ep,request(1,1,0,0)).await.unwrap().vote_granted);
    file(&restarted,[2,1,1,1,0,0],"new requests use fresh operation IDs").await;
    rt.kill(id);
}
#[madsim::test]
async fn restart_confirmed() { restart_case(false).await; }
#[madsim::test]
async fn restart_written_pending() { restart_case(true).await; }
#[madsim::test]
async fn malformed_storage_is_error() {
    let node=Handle::current().create_node().build();
    node.spawn(async {
        let c=Config{node:0,members:3};
        assert!(vote_host::load(c,true).await.is_err());
        for bytes in [vec![],vec![0;47],vec![0;49],vec![255;48]] {
            fs::File::create("state").await.unwrap().write_all_at(&bytes,0).await.unwrap();
            assert!(vote_host::load(c,true).await.is_err(),"bad encoding must not become fresh state");
        }
    }).await.unwrap();
}
