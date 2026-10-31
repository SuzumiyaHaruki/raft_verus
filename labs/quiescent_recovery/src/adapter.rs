//! UNVERIFIED host wiring, adapted locally from async_completion. Recovery only
//! applies after the last accepted operation succeeded and its client got Ack.
use crate::core::{Ack, Begin, Completion, Core, Peer, Put, Record, Write};
use async_channel::{Receiver, Sender};
use madsim::{fs, net::Endpoint};
use std::{io, net::{Ipv4Addr, SocketAddr}};

pub const PUT: u64 = 1;
pub const ACK: u64 = 2;
#[derive(Clone, Copy)]
pub enum Mode { Fresh, Recover }
#[derive(Clone, Copy, Debug)]
pub struct View { pub value: u64, pub next: u64, pub pending: bool }
pub struct Hooks { pub started: Sender<View>, pub parked: Sender<View> }
pub struct IoControl { pub entered: Sender<(Write, View)>, pub release: Receiver<()> }

fn view(core: &Core) -> View {
    View { value: core.value(), next: core.next_id(), pending: core.pending().is_some() }
}

async fn recover_file(path: &str) -> io::Result<Core> {
    let bytes = fs::read(path).await?; // Missing/read errors NEVER mean fresh start.
    if bytes.len() != 16 { return Err(io::Error::new(io::ErrorKind::InvalidData, "record needs 16 bytes")); }
    let record = Record {
        id: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
        value: u64::from_le_bytes(bytes[8..].try_into().unwrap()),
    };
    Core::recover(record).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "illegal record ID"))
}

pub async fn load_core(path: &str, mode: Mode) -> io::Result<Core> {
    match mode {
        Mode::Recover => recover_file(path).await,
        Mode::Fresh => match fs::read(path).await {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Core::new()),
            Err(e) => Err(e),
            Ok(_) => Err(io::Error::new(io::ErrorKind::AlreadyExists, "fresh start requires empty storage")),
        },
    }
}

async fn write_record(path: &str, write: Write) -> io::Result<()> {
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&write.id.to_le_bytes());
    bytes[8..].copy_from_slice(&write.request.value.to_le_bytes());
    let file = fs::File::create(path).await?;
    file.write_all_at(&bytes, 0).await
}

async fn send_ack(endpoint: &Endpoint, ack: Ack) -> io::Result<()> {
    let destination = SocketAddr::from((Ipv4Addr::from(ack.destination.ip), ack.destination.port));
    endpoint.send_to(destination, ACK, &ack.value.to_le_bytes()).await
}

/// A bounded service for the controlled restart experiment. After `count`
/// requests it parks with its Core/Endpoint inside this task, accepting no more.
/// Hooks only report state/control scheduling; they supply no recovery values.
pub async fn serve(address: SocketAddr, path: &str, mode: Mode, count: usize,
    hooks: Hooks, control: Option<IoControl>) -> io::Result<()>
{
    let mut core = load_core(path, mode).await?;
    let endpoint = Endpoint::bind(address).await?;
    hooks.started.send(view(&core)).await.map_err(|_| io::Error::other("observer closed"))?;
    for _ in 0..count {
        let mut bytes = [0u8; 9];
        let (length, source) = endpoint.recv_from(PUT, &mut bytes).await?;
        if length != 8 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Put needs 8 bytes")); }
        let SocketAddr::V4(source) = source else { return Err(io::Error::other("IPv4 only")); };
        let request = Put { source: Peer { ip: u32::from(*source.ip()), port: source.port() },
            value: u64::from_le_bytes(bytes[..8].try_into().unwrap()) };
        let write = match core.begin(request) {
            Begin::Write(w) => w,
            Begin::Busy => return Err(io::Error::other("unexpected Busy in sequential adapter")),
            Begin::Exhausted => return Err(io::Error::other("operation IDs exhausted")),
        };
        if let Some(control) = &control {
            control.entered.send((write, view(&core))).await.map_err(|_| io::Error::other("observer closed"))?;
            control.release.recv().await.map_err(|_| io::Error::other("release closed"))?;
        }
        let result = write_record(path, write).await;
        let reply = core.complete(Completion { id: write.id, success: result.is_ok() });
        if let Some(ack) = reply { send_ack(&endpoint, ack).await?; }
        result?; // Failed-operation recovery is outside this experiment's scope.
    }
    hooks.parked.send(view(&core)).await.map_err(|_| io::Error::other("observer closed"))?;
    std::future::pending::<()>().await;
    drop(core);
    drop(endpoint);
    Ok(())
}
