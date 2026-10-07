use std::{
    io::{self, Write},
    net::{Ipv4Addr, Shutdown, TcpListener, TcpStream},
    sync::atomic::{AtomicU32, Ordering},
    thread,
};

use crate::protocol::{self, Token};

pub struct ControlServer {
    listener: TcpListener,
}

impl ControlServer {
    pub fn bind(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))?;
        Ok(Self { listener })
    }

    pub fn run(&self, session_token: &AtomicU32) {
        let mut current: Option<TcpStream> = None;

        thread::scope(|s| {
            for stream in self.listener.incoming() {
                let stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("accept: {e}");
                        continue;
                    }
                };

                let handle = match stream.try_clone() {
                    Ok(handle) => handle,
                    Err(e) => {
                        eprintln!("clone stream: {e}");
                        continue;
                    }
                };

                if let Some(old) = current.replace(handle) {
                    eprintln!("new phone connected, replacing the current session");
                    let _ = old.shutdown(Shutdown::Both);
                }

                s.spawn(move || {
                    let result =
                        Session::new(stream, session_token).and_then(|session| session.run());
                    if let Err(e) = result {
                        eprintln!("session ended: {e}");
                    }
                });
            }
        });
    }
}

struct Session<'a> {
    stream: TcpStream,
    session_token: &'a AtomicU32,
}

impl<'a> Session<'a> {
    fn new(stream: TcpStream, session_token: &'a AtomicU32) -> io::Result<Self> {
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            session_token,
        })
    }

    fn run(mut self) -> io::Result<()> {
        let peer = self.stream.peer_addr()?;
        let token = protocol::new_token()?;

        // Publish before the phone learns the token, or its first packets get dropped.
        self.session_token.store(token, Ordering::Release);
        let _guard = TokenGuard {
            slot: self.session_token,
            token,
        };

        self.stream.write_all(&token.to_le_bytes())?;
        eprintln!("session started with {peer}");

        io::copy(&mut self.stream, &mut io::sink())?;
        eprintln!("session with {peer} closed");
        Ok(())
    }
}

struct TokenGuard<'a> {
    slot: &'a AtomicU32,
    token: Token,
}

impl Drop for TokenGuard<'_> {
    fn drop(&mut self) {
        // A newer session may have replaced ours, so only clear our own token.
        let _ = self.slot.compare_exchange(
            self.token,
            protocol::NO_SESSION,
            Ordering::AcqRel,
            Ordering::Relaxed,
        );
    }
}
