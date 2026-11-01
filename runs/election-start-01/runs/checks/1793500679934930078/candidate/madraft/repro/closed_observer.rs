//! Reproduce the protected restart observer's closed-channel outcome.
//! This is diagnostic code only; it is not imported by Cargo or the core proof.
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};
fn main() {
    let (done, observed) = async_channel::bounded::<()>(1);
    // The protected test moves its sole sender into the task it aborts and joins.
    let cancelled_task = async move { done.send(()).await.unwrap(); };
    drop(cancelled_task);
    let mut receive = pin!(observed.recv());
    let mut cx = Context::from_waker(Waker::noop());
    match receive.as_mut().poll(&mut cx) {
        Poll::Ready(Err(_)) => println!("READY_CLOSED: no message was sent; recv completes with Err, so timeout returns Ok(Err), not Err(elapsed)"),
        _ => panic!("expected the cancelled sole sender to close the empty channel"),
    }
}
