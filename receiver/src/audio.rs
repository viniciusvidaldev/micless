use std::{
    io,
    net::{Ipv4Addr, UdpSocket},
    sync::atomic::{AtomicU32, Ordering},
};

use crate::protocol::{self, AudioPacket};

pub struct AudioReceiver {
    socket: UdpSocket,
}

impl AudioReceiver {
    pub fn bind(port: u16) -> io::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?;
        Ok(Self { socket })
    }

    pub fn run(&self, session_token: &AtomicU32, ring: &mut ring::Producer) -> io::Result<()> {
        let mut buf = [0u8; protocol::MAX_DATAGRAM];
        let mut samples = [0i16; protocol::SAMPLES_PER_PACKET];

        loop {
            let n = match self.socket.recv(&mut buf) {
                Ok(n) => n,
                Err(e) if is_retryable(&e) => continue,
                Err(e) => return Err(e),
            };

            let Some(packet) = AudioPacket::parse(&buf[..n]) else {
                continue;
            };

            let expected = session_token.load(Ordering::Acquire);
            if expected == protocol::NO_SESSION || packet.token != expected {
                continue;
            }

            let Some(count) = packet.decode(&mut samples) else {
                continue;
            };

            ring.push(&samples[..count]);
        }
    }
}

fn is_retryable(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}
