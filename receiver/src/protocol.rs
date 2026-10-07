use std::{
    fs::File,
    io::{self, Read},
};

pub const CONTROL_PORT: u16 = 47300;
pub const AUDIO_PORT: u16 = 47300;

pub const SAMPLES_PER_PACKET: usize = 480;

// Audio datagram: [token u32 le][seq u32 le][samples i16 le]
pub const TOKEN_LEN: usize = 4;
pub const SEQ_LEN: usize = 4;
pub const HEADER_LEN: usize = TOKEN_LEN + SEQ_LEN;

pub const MAX_DATAGRAM: usize = HEADER_LEN + SAMPLES_PER_PACKET * size_of::<i16>();

pub type Token = u32;

pub const NO_SESSION: Token = 0;

/// One audio datagram, borrowed from the receive buffer.
pub struct AudioPacket<'a> {
    pub token: Token,
    pub seq: u32,
    pub payload: &'a [u8],
}

impl<'a> AudioPacket<'a> {
    pub fn parse(datagram: &'a [u8]) -> Option<Self> {
        let (token, rest) = datagram.split_first_chunk::<TOKEN_LEN>()?;
        let (seq, payload) = rest.split_first_chunk::<SEQ_LEN>()?;
        Some(Self {
            token: Token::from_le_bytes(*token),
            seq: u32::from_le_bytes(*seq),
            payload,
        })
    }

    /// Writes the samples into `out` and returns how many there were.
    pub fn decode(&self, out: &mut [i16]) -> Option<usize> {
        let count = self.payload.len() / 2;
        if !self.payload.len().is_multiple_of(2) || count > out.len() {
            return None;
        }
        for (i, pair) in self.payload.chunks_exact(2).enumerate() {
            out[i] = i16::from_le_bytes([pair[0], pair[1]]);
        }
        Some(count)
    }
}

/// Never returns `NO_SESSION`.
pub fn new_token() -> io::Result<Token> {
    let mut f = File::open("/dev/urandom")?;
    loop {
        let mut buf = [0u8; TOKEN_LEN];
        f.read_exact(&mut buf)?;
        let token = Token::from_le_bytes(buf);
        if token != NO_SESSION {
            return Ok(token);
        }
    }
}
