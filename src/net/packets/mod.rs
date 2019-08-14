use bytes::{Bytes, BytesMut};
use std::{convert::TryFrom, io};

mod bufext;
mod handshake;
mod login;
mod parse;
mod status;

pub use self::{bufext::*, handshake::*, login::*, status::*};

#[derive(Clone, Debug)]
pub enum IncomingPackets {
    EncryptionResponse(EncryptionResponse),
    Handshake(Handshake),
    LoginStart(LoginStart),
    Ping(Ping),
    StatusHandshake(StatusHandshake),
}

#[derive(Clone, Debug)]
pub enum OutgoingPackets {
    EncryptionRequest(EncryptionRequest),
    Disconnect(Disconnect),
    Ping(Ping),
    StatusResponse(StatusResponse),
}

pub trait Incoming: TryFrom<Bytes, Error = io::Error> {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    fn validate_self(self) -> Result<Self, String> {
        match self.validate() {
            Ok(_) => Ok(self),
            Err(e) => Err(e),
        }
    }
}

pub trait Outgoing {
    fn written_len(&self) -> usize;

    fn write_to(&self, dst: &mut BytesMut) -> Result<(), io::Error>;
}

macro_rules! into_getter {
    ($($name:ident -> $variant:ident),+) => {
        $(
            pub fn $name(self) -> Result<$variant, Self> {
                match self {
                    Self::$variant(x) => Ok(x),
                    _ => Err(self)
                }
            }
        )*
    };
}

// TODO: Generate this using a proc macro
impl IncomingPackets {
    into_getter!(
        into_encryption_response -> EncryptionResponse,
        into_handshake -> Handshake,
        into_login_start -> LoginStart,
        into_ping -> Ping,
        into_status_handshake -> StatusHandshake
    );
}
