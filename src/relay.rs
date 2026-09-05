use std::net::{SocketAddr, UdpSocket};

use tracing::{debug, info, warn};

/// Length of the 0xFF synchronization stream at the start of a magic packet
const SYNC_STREAM_LEN: usize = 6;
/// Length of a single MAC address (in bytes)
const MAC_LEN: usize = 6;
/// Number of times the target MAC is repeated in a canonical `WoL` magic packet
const MAC_REPEAT_COUNT: usize = 16;
/// Minimum length of a `WoL` magic packet: sync + repetitions × MAC
const MIN_WOL_PACKET_LEN: usize = SYNC_STREAM_LEN + MAC_REPEAT_COUNT * MAC_LEN;

pub fn run(listen: SocketAddr, broadcast: SocketAddr) -> Result<(), std::io::Error> {
    let listener = UdpSocket::bind(listen)?;
    let sender = UdpSocket::bind("0.0.0.0:0")?;
    sender.set_broadcast(true)?;

    info!("wakeonlan-relay listening on {listen}, forwarding to {broadcast}");

    let mut buf = [0u8; 2048];
    loop {
        let (size, source) = match listener.recv_from(&mut buf) {
            Ok(t) => t,
            Err(e) => {
                warn!("recv error: {e}");
                continue;
            }
        };

        debug!("received {size} bytes from {source}");

        if size < MIN_WOL_PACKET_LEN {
            debug!("ignored {size}-byte packet from {source}: too short");
            continue;
        }

        // Magic packet starts with SYNC_STREAM_LEN × 0xFF
        let is_magic = buf
            .get(..size)
            .and_then(|p| p.get(..SYNC_STREAM_LEN))
            .is_some_and(|hdr| hdr.iter().all(|&b| b == 0xFF));
        if !is_magic {
            debug!("ignored packet from {source}: not a WoL packet");
            continue;
        }

        let Some(payload) = buf.get(..size) else {
            continue;
        };
        match sender.send_to(payload, broadcast) {
            Ok(sent) => info!("forwarded {sent} bytes to {broadcast}"),
            Err(e) => warn!("send error: {e}"),
        }
    }
}
