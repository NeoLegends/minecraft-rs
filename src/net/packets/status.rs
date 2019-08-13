use super::{
    bufext::{var_i64_length, var_usize_length, VarReadExt, VarWriteExt},
    Incoming, Outgoing,
};
use bytes::{Bytes, BytesMut, IntoBuf};
use serde_json::json;
use std::{convert::TryFrom, io};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ping {
    pub value: i64,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct StatusHandshake(());

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StatusResponse {
    pub version: String,
    pub protocol_version: u32,
    pub players_max: usize,
    pub players_online: usize,
    pub description: String,
    pub favicon: String,
}

impl TryFrom<Bytes> for Ping {
    type Error = io::Error;

    fn try_from(data: Bytes) -> io::Result<Self> {
        let value = data.into_buf().read_var_i64()?;
        Ok(Ping { value })
    }
}

impl Incoming for Ping {}

impl Outgoing for Ping {
    fn written_len(&self) -> usize {
        var_i64_length(self.value)
    }

    fn write_to(&self, dst: &mut BytesMut) -> io::Result<()> {
        dst.write_var_i64(self.value)
    }
}

impl TryFrom<Bytes> for StatusHandshake {
    type Error = io::Error;

    fn try_from(_: Bytes) -> io::Result<Self> {
        Ok(StatusHandshake(()))
    }
}

impl Incoming for StatusHandshake {}

impl StatusResponse {
    fn build_json(&self) -> String {
        let json = json!({
            "version": {
                "name": self.version,
                "protocol": self.protocol_version
            },
            "players": {
                "max": self.players_max,
                "online": self.players_online,
                "sample": []
            },
            "description": {
                "text": self.description
            },
            "favicon": format!("data:image/png;base64,{}", self.favicon)
        });

        json.to_string()
    }
}

impl Outgoing for StatusResponse {
    fn written_len(&self) -> usize {
        let stringified = self.build_json();

        var_usize_length(stringified.len()) + stringified.len()
    }

    fn write_to(&self, dst: &mut BytesMut) -> io::Result<()> {
        let stringified = self.build_json();

        dst.reserve(5 + stringified.len());
        dst.write_str(&stringified)
    }
}
