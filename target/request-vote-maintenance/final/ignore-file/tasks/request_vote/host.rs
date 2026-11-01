//! UNVERIFIED I/O/serialization/serialization-of-operations boundary.
//! No voting eligibility, term changes or reply authorization belongs here.
use super::{vote_api::*, vote_core::Core};
use async_channel::{Receiver, Sender};
use madsim::fs;
use std::io;
use std::convert::TryInto;

#[derive(Clone)]
pub struct Barrier { pub op: u64, pub after_write: bool,
    pub entered: Sender<Write>, pub release: Receiver<()> }
#[derive(Clone, Default)]
pub struct Hooks { pub barrier: Option<Barrier>, pub fail_write: bool }

pub fn encode(record: Record) -> Vec<u8> {
    let words = [record.op, record.state.term, u64::from(record.state.voted_for.is_some()),
        record.state.voted_for.unwrap_or(0), record.state.log.term, record.state.log.index];
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}
pub fn decode(bytes: &[u8]) -> io::Result<Record> {
    if bytes.len() != 48 { return Err(io::Error::new(io::ErrorKind::InvalidData,"record length")); }
    let w: Vec<u64> = bytes.chunks_exact(8).map(|x| u64::from_le_bytes(x.try_into().unwrap())).collect();
    let vote = match w[2] { 0 if w[3] == 0 => None, 1 => Some(w[3]),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData,"vote tag")) };
    Ok(Record { op:w[0], state:Persistent{term:w[1],voted_for:vote,log:Log{term:w[4],index:w[5]}} })
}
pub async fn load(config: Config, recovery_only: bool) -> io::Result<Core> {
    let core = match fs::read("state").await {
        Ok(_bytes) => Core::new(config, Initial { state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower }), // DEFECT: ignores retained file
        Err(e) if e.kind() == io::ErrorKind::NotFound && !recovery_only => Core::new(config,
            Initial { state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower }),
        Err(e) => return Err(e),
    };
    core.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,"core rejected state/config"))
}
pub async fn checkpoint(h: &Hooks, w: Write, after_write: bool) -> io::Result<()> {
    if let Some(b) = &h.barrier {
        if b.op == w.record.op && b.after_write == after_write {
            b.entered.send(w).await.map_err(|_| io::Error::other("observer closed"))?;
            b.release.recv().await.map_err(|_| io::Error::other("barrier closed"))?;
        }
    }
    Ok(())
}
pub async fn save(w: Write, h: &Hooks) -> io::Result<()> {
    checkpoint(h,w,false).await?;
    let file = fs::File::create("state").await?;
    // Fault injection uses a real read-only file handle: the actual write fails.
    let file = if h.fail_write { fs::File::open("state").await? } else { file };
    file.write_all_at(&encode(w.record),0).await?;
    file.sync_all().await?;
    checkpoint(h,w,true).await?;
    Ok(())
}
