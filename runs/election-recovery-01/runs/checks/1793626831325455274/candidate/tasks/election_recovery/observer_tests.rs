// Test-only observation. Quiet requires an actual live producer; Closed is
// cleanup, and every received value (including an error) ends observation.
#[derive(Debug)]
enum Observation<T> { Quiet, Closed, Received(T) }
async fn observe<T>(receiver: &async_channel::Receiver<T>) -> Observation<T> {
    match timeout(Duration::from_millis(30),receiver.recv()).await {
        Err(_) => Observation::Quiet,
        Ok(Err(_)) => Observation::Closed,
        Ok(Ok(value)) => Observation::Received(value),
    }
}
fn assert_quiet<T: std::fmt::Debug>(result: Observation<T>, label: &str) {
    assert!(matches!(result,Observation::Quiet),"{}: observation ended or received output: {:?}",label,result);
}
#[madsim::test]
async fn observer_live_then_cancelled() {
    let (send,recv)=async_channel::bounded::<()>(1);
    let (entered,ready)=async_channel::bounded(1);
    let worker=task::spawn(async move {
        entered.send(()).await.unwrap();
        std::future::pending::<()>().await;
        send.send(()).await.unwrap();
    });
    bounded(ready.recv()).await.unwrap();
    assert_quiet(observe(&recv).await,"live worker has no output in window");
    worker.abort();
    assert!(bounded(worker).await.unwrap_err().is_cancelled());
    assert!(matches!(observe(&recv).await,Observation::Closed));
    println!("abort/join sole sender: Ok(Err(RecvError)), not Err(Elapsed); cleanup only");
}
#[madsim::test]
async fn observer_queued_reply_then_closed() {
    let (send,recv)=async_channel::bounded(1);
    send.send(Ok::<_,String>((1u64,true))).await.unwrap(); drop(send);
    let result=observe(&recv).await;
    assert!(matches!(result,Observation::Received(Ok((1,true)))));
    assert!(std::panic::catch_unwind(|| assert_quiet(result,"queued protocol reply")).is_err());
    assert!(matches!(observe(&recv).await,Observation::Closed));
}
#[madsim::test]
async fn observer_transport_error() {
    let node=Handle::current().create_node().ip(peers()[1].ip()).build();
    let (send,recv)=async_channel::bounded(1);
    let worker=node.spawn(async move {
        let ep=Endpoint::bind(peers()[1]).await.unwrap();
        let result=ep.call_timeout::<RequestVoteArgs>(peers()[0],RequestVoteArgs{
            term:1,candidate_id:1,last_log_term:0,last_log_index:0},Duration::from_millis(1)).await;
        send.send(result.map_err(|e|e.kind())).await.unwrap();
    });
    bounded(worker).await.unwrap();
    let result=observe(&recv).await;
    assert!(matches!(result,Observation::Received(Err(_))),"real RPC error must remain distinct");
    assert!(std::panic::catch_unwind(|| assert_quiet(result,"transport failure")).is_err());
}
#[madsim::test]
async fn observer_panicked_producer() {
    use futures::FutureExt;
    let (send,recv)=async_channel::bounded::<()>(1);
    // Catch at the task boundary so the runtime can inspect the closed channel.
    let worker=task::spawn(std::panic::AssertUnwindSafe(async move {
        let _owner=send;
        panic!("controlled observer panic");
    }).catch_unwind());
    assert!(bounded(worker).await.unwrap().is_err());
    let result=observe(&recv).await;
    assert!(matches!(result,Observation::Closed));
    assert!(std::panic::catch_unwind(|| assert_quiet(result,"observer died")).is_err());
}
