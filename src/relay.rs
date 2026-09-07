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

type Mac = [u8; MAC_LEN];

pub fn parse_magic_packet(packet: &[u8]) -> Option<Mac> {
    let body = packet.get(..MIN_WOL_PACKET_LEN)?;
    let (sync, repeats) = body.split_at(SYNC_STREAM_LEN);
    if !sync.iter().all(|&b| b == 0xFF) {
        return None;
    }
    let mac: Mac = repeats.get(..MAC_LEN)?.try_into().ok()?;
    repeats
        .as_chunks::<MAC_LEN>()
        .0
        .iter()
        .all(|&chunk| chunk == mac)
        .then_some(mac)
}

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
        let Some(packet) = buf.get(..size) else {
            continue;
        };
        let Some(mac) = parse_magic_packet(packet) else {
            debug!("ignored {size} bytes packet from {source}: not a WoL packet");
            continue;
        };
        match sender.send_to(packet, broadcast) {
            Ok(sent) => info!("forwarded {sent} bytes to {broadcast} for {mac:02X?}"),
            Err(e) => warn!("send error: {e}"),
        }
    }
}
