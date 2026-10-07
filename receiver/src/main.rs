use std::{
    io,
    net::{IpAddr, Ipv4Addr, UdpSocket},
    sync::atomic::AtomicU32,
    thread,
};

use crate::{audio::AudioReceiver, control::ControlServer};

mod audio;
mod control;
mod protocol;

fn main() -> io::Result<()> {
    let control = ControlServer::bind(protocol::CONTROL_PORT)?;
    let receiver = AudioReceiver::bind(protocol::AUDIO_PORT)?;
    let mut ring = open_ring()?;
    let session_token = AtomicU32::new(protocol::NO_SESSION);

    println!("micless receiver ready");
    println!("enter this address in the phone app: {}", local_ip()?);

    thread::scope(|s| {
        s.spawn(|| {
            if let Err(e) = receiver.run(&session_token, &mut ring) {
                eprintln!("audio receiver stopped: {e}");
            }
        });
        control.run(&session_token);
    });

    Ok(())
}

/// The address the phone should connect to: the one the default route goes out of.
fn local_ip() -> io::Result<IpAddr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    // connect on UDP sends nothing, it only makes the kernel pick a route and source address.
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80))?;
    Ok(socket.local_addr()?.ip())
}

fn open_ring() -> io::Result<ring::Producer> {
    match ring::Producer::open() {
        Ok(ring) => Ok(ring),
        Err(e) => {
            eprintln!("ring unusable ({e}), recreating");
            ring::Producer::unlink()?;
            ring::Producer::open()
        }
    }
}
