//! UNVERIFIED: source/byte conversion, exclusive driving, truthful asynchronous
//! I/O execution/result reporting, and unchanged Ack forwarding.
use crate::core::{Ack, Begin, Completion, Core, Peer, Put, Write};
use async_channel::{Receiver, Sender};
use madsim::{fs, net::Endpoint};
use std::{io, net::{Ipv4Addr, SocketAddr}};

pub const PUT: u64 = 1;
pub const ACK: u64 = 2;

// Optional narrow test boundary: Continue still performs the real write.
// Fail injects an explicit error before file creation, not a disk fault model.
pub enum Release { Continue, Fail }
pub struct IoControl { pub entered: Sender<Write>, pub release: Receiver<Release> }
pub struct Exit { pub core: Core, pub write_error: Option<io::Error> }

fn source_peer(source: SocketAddr) -> io::Result<Peer> {
    match source {
        SocketAddr::V4(v4) => Ok(Peer { ip: u32::from(*v4.ip()), port: v4.port() }),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "IPv4 only")),
    }
}

async fn write_record(path: &str, write: Write, control: Option<&IoControl>) -> io::Result<()> {
    if let Some(control) = control {
        control.entered.send(write).await.map_err(|_| io::Error::other("observer closed"))?;
        match control.release.recv().await.map_err(|_| io::Error::other("release closed"))? {
            Release::Continue => {},
            Release::Fail => return Err(io::Error::other("injected write failure before create")),
        }
    }
    let mut record = [0u8; 16];
    record[..8].copy_from_slice(&write.id.to_le_bytes());
    record[8..].copy_from_slice(&write.request.value.to_le_bytes());
    let file = fs::File::create(path).await?;
    file.write_all_at(&record, 0).await
}

async fn send_ack(endpoint: &Endpoint, ack: Ack) -> io::Result<()> {
    let destination = SocketAddr::from((Ipv4Addr::from(ack.destination.ip), ack.destination.port));
    endpoint.send_to(destination, ACK, &ack.value.to_le_bytes()).await
}

/// One owner; one in-flight operation. No retry or cancellation-time Ack.
/// Write failure is fed to the core, then returned with the inspectable state.
pub async fn serve(endpoint: Endpoint, path: &str, count: usize, control: Option<IoControl>) -> io::Result<Exit> {
    let mut core = Core::new();
    for _ in 0..count {
        let mut bytes = [0u8; 9];
        let (length, source) = endpoint.recv_from(PUT, &mut bytes).await?;
        if length != 8 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Put needs eight bytes")); }
        let request = Put { source: source_peer(source)?, value: u64::from_le_bytes(bytes[..8].try_into().unwrap()) };
        let write = match core.begin(request) {
            Begin::Write(write) => write,
            Begin::Busy => return Err(io::Error::other("unexpected Busy in sequential adapter")),
            Begin::Exhausted => return Err(io::Error::other("operation IDs exhausted")),
        };
        let result = write_record(path, write, control.as_ref()).await;
        let reply = core.complete(Completion { id: write.id, success: result.is_ok() });
        if let Some(ack) = reply { send_ack(&endpoint, ack).await?; }
        if let Err(error) = result { return Ok(Exit { core, write_error: Some(error) }); }
    }
    Ok(Exit { core, write_error: None })
}
