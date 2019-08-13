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
}

pub trait Outgoing {
    fn written_len(&self) -> usize;

    fn write_to(&self, dst: &mut BytesMut) -> Result<(), io::Error>;
}
