mod codec;
mod handshake;
mod login;
mod status;

pub use self::{codec::Coder, handshake::*, login::*, status::*};

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
