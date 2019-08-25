use super::Incoming;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Deserialize_repr)]
#[repr(u8)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Deserialize)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_addr: String,
    pub server_port: u16,
    pub next_state: NextState,
}

impl Incoming for Handshake {
    fn validate(&self) -> Result<(), String> {
        match self.protocol_version {
            404 => Ok(()),
            _ => Err("Unknown protocol version".to_owned()),
        }
    }
}
