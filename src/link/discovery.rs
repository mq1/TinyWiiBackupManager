// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Finding companion apps in the local network; blocking, run it with smol::unblock

use crate::link::protocol::{Info, LEGACY_QUERY, QUERY, command, read_answer, write_request};
use std::{
    collections::HashMap,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket},
    thread,
    time::{Duration, Instant},
};

// a 3DS in power saving misses broadcasts, so ask again and again
const ASK_EVERY: Duration = Duration::from_millis(400);
const SCAN_TIMEOUT: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub addr: SocketAddr,
    pub info: Info,
}

/// The companion apps that answer within `timeout`; `known` are addresses seen before
pub fn discover(timeout: Duration, known: &[IpAddr], port: u16) -> io::Result<Vec<Found>> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_broadcast(true)?;

    let mut targets: Vec<IpAddr> = vec![Ipv4Addr::BROADCAST.into()];
    if let Some(local) = local_ipv4() {
        let [a, b, c, _] = local.octets();
        targets.push(Ipv4Addr::new(a, b, c, 255).into());
    }
    targets.extend_from_slice(known);

    let deadline = Instant::now() + timeout;
    let mut next_ask = Instant::now();
    let mut found: HashMap<IpAddr, Found> = HashMap::new();
    let mut buf = [0; 2048];

    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        if Instant::now() >= next_ask {
            for target in &targets {
                for query in [QUERY, LEGACY_QUERY] {
                    // some targets (e.g. the broadcast address) may be refused; the others count
                    let _ = socket.send_to(query, (*target, port));
                }
            }
            next_ask = Instant::now() + ASK_EVERY;
        }

        let wait = left
            .min(next_ask.saturating_duration_since(Instant::now()))
            .max(Duration::from_millis(10));
        socket.set_read_timeout(Some(wait))?;

        match socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if let Some(info) = Info::parse_discovery_reply(&buf[..n]) {
                    let addr = SocketAddr::new(from.ip(), info.port());
                    // a companion that knows both queries answers twice; keep the newer reply
                    let newer = found
                        .get(&from.ip())
                        .is_none_or(|f| f.info.proto() <= info.proto());
                    if newer {
                        found.insert(from.ip(), Found { addr, info });
                    }
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(found.into_values().collect())
}

/// When nobody answers discovery (some routers drop broadcasts): tries every address of the /24
pub fn scan(port: u16) -> Vec<Found> {
    let Some(local) = local_ipv4() else {
        return Vec::new();
    };
    let [a, b, c, _] = local.octets();

    let handles: Vec<_> = (1..=254)
        .map(|d| {
            let addr = SocketAddr::new(Ipv4Addr::new(a, b, c, d).into(), port);
            thread::spawn(move || probe(addr))
        })
        .collect();

    handles
        .into_iter()
        .filter_map(|handle| handle.join().ok().flatten())
        .collect()
}

fn probe(addr: SocketAddr) -> Option<Found> {
    let mut stream = TcpStream::connect_timeout(&addr, SCAN_TIMEOUT).ok()?;
    stream.set_read_timeout(Some(SCAN_TIMEOUT * 4)).ok()?;
    write_request(&mut stream, command::INFO, &[]).ok()?;
    let text = read_answer(&mut stream).ok()?;
    let info = Info::parse(&String::from_utf8_lossy(&text));

    (!info.app().is_empty()).then_some(Found { addr, info })
}

/// The address of this computer in the local network (no packet is sent)
fn local_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).ok()?;

    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => Some(ip),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_companion_that_was_seen_before() {
        let companion = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = companion.local_addr().unwrap().port();

        thread::spawn(move || {
            let mut buf = [0; 64];
            loop {
                let Ok((n, from)) = companion.recv_from(&mut buf) else {
                    return;
                };
                let reply: &[u8] = match &buf[..n] {
                    b"ugbm?" => b"ugbm-link\nproto=1\napp=ugbm-link-wii\nconsole=wii\n",
                    b"vcinject?" => b"vcinject-link\napp=vcinject-link\n",
                    _ => continue,
                };
                let _ = companion.send_to(reply, from);
            }
        });

        let found = discover(
            Duration::from_millis(500),
            &["127.0.0.1".parse().unwrap()],
            port,
        )
        .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].info.app(), "ugbm-link-wii");
        assert_eq!(found[0].addr.ip(), IpAddr::from(Ipv4Addr::LOCALHOST));
    }
}
