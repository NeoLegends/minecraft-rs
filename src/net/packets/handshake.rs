use super::Incoming;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Deserialize)]
pub enum NextState {
    Status,
    Login,
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
