//! Exact abort/join/timeout diagnostic on the pinned MadSim runtime.
//! No Raft or protected test code is modified or substituted.
use madsim::{runtime::Runtime, task, time::{timeout, Duration}};
fn main() {
    for seed in 1..=5 {
        Runtime::with_seed_and_config(seed, Default::default()).block_on(async move {
            let (done, observed) = async_channel::bounded::<()>(1);
            let (entered, waiting) = async_channel::bounded(1);
            let running = task::spawn(async move {
                entered.send(()).await.unwrap();
                std::future::pending::<()>().await;
                done.send(()).await.unwrap();
            });
            timeout(Duration::from_secs(3), waiting.recv()).await.unwrap().unwrap();
            // Like restart_case, cancel and await the task owning the sole sender.
            running.abort();
            let joined = timeout(Duration::from_secs(3), running).await.unwrap();
            assert!(joined.unwrap_err().is_cancelled());
            assert!(observed.is_closed());
            assert!(observed.is_empty());
            let result = timeout(Duration::from_millis(30), observed.recv()).await;
            assert!(matches!(result, Ok(Err(_))));
            println!("seed {seed}: cancelled task joined; empty closed observer; timeout = Ok(Err(RecvError)), no message");
        });
    }
}
