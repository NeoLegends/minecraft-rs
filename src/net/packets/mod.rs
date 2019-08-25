mod codec;
mod handshake;
mod login;
mod status;

pub use self::{codec::Coder, handshake::*, login::*, status::*};

#[derive(Clone, Debug, enum_as_inner::EnumAsInner)]
pub enum IncomingPackets {
    EncryptionResponse(EncryptionResponse),
    Handshake(Handshake),
    LoginStart(LoginStart),
    Ping(Ping),
    StatusHandshake(StatusHandshake),
}

#[derive(Clone, Debug, enum_as_inner::EnumAsInner)]
pub enum OutgoingPackets {
    EncryptionRequest(EncryptionRequest),
    Disconnect(Disconnect),
    LoginSuccess(LoginSuccess),
    Ping(Ping),
    StatusResponse(StatusResponse),
}

pub trait Incoming {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    fn validate_self(self) -> Result<Self, String>
    where
        Self: Sized,
    {
        match self.validate() {
            Ok(_) => Ok(self),
            Err(e) => Err(e),
        }
    }
}
