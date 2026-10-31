//! TRUSTED, UNVERIFIED adapter: actual-source decoding, exclusive ownership,
//! unmodified core invocation/action forwarding, address and u64 byte codecs.
use crate::core::{Core, Peer, ReceivedPut, SendAck};
use madsim::net::Endpoint;
use std::{io, net::{Ipv4Addr, SocketAddr}};

pub const PUT: u64 = 1;
pub const ACK: u64 = 2;

fn source_peer(source: SocketAddr) -> io::Result<Peer> {
    match source {
        SocketAddr::V4(v4) => Ok(Peer { ip: u32::from(*v4.ip()), port: v4.port() }),
        SocketAddr::V6(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "IPv4 only")),
    }
}

fn address(peer: Peer) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::from(peer.ip), peer.port))
}

async fn send_action(endpoint: &Endpoint, action: SendAck) -> io::Result<()> {
    let destination = address(action.destination);
    let bytes = action.value.to_le_bytes();
    endpoint.send_to(destination, ACK, &bytes).await
}

/// One task owns the core and handles this many successfully decoded requests.
/// A send error stops this task without undoing a processed Put. It says
/// nothing about whether that send emitted a packet before reporting failure.
pub async fn serve(endpoint: Endpoint, requests: usize) -> io::Result<u64> {
    let mut core = Core::new();
    for _ in 0..requests {
        // The extra byte detects oversize/truncated datagrams as invalid.
        let mut bytes = [0u8; 9];
        let (length, source) = endpoint.recv_from(PUT, &mut bytes).await?;
        if length != 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Put needs eight bytes"));
        }
        let request = ReceivedPut {
            source: source_peer(source)?,
            value: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
        };
        let action = core.handle(request);
        send_action(&endpoint, action).await?;
    }
    Ok(core.value())
}
