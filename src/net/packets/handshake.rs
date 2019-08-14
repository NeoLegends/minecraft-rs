use super::{bufext::VarReadExt, Incoming};
use bytes::{Buf, Bytes};
use std::{
    convert::TryFrom,
    io::{self, Cursor, ErrorKind},
};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum NextState {
    Login,
    Status,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_addr: String,
    pub server_port: u16,
    pub next_state: NextState,
}

impl TryFrom<Bytes> for Handshake {
    type Error = io::Error;

    fn try_from(data: Bytes) -> io::Result<Self> {
        let mut cur = Cursor::new(data);

        let protocol_version = cur.read_var_i32()?;
        let server_addr = cur.read_str()?;
        let server_port = cur.get_u16_be();
        let next_state = match cur.read_var_i32()? {
            1 => NextState::Status,
            2 => NextState::Login,
            _ => return Err(ErrorKind::InvalidData.into()),
        };

        Ok(Handshake {
            protocol_version,
            server_addr,
            server_port,
            next_state,
        })
    }
}

impl Incoming for Handshake {
    fn validate(&self) -> Result<(), String> {
        match self.protocol_version {
            404 => Ok(()),
            _ => Err("Unknown protocol version".to_owned()),
        }
    }
}
