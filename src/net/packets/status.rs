use super::{
    bufext::{var_i64_length, var_usize_length, VarReadExt, VarWriteExt},
    Incoming, Outgoing,
};
use bytes::{Bytes, BytesMut, IntoBuf};
use serde_json::{json, Value};
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
    pub favicon: Option<String>,
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
        let mut json = json!({
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
        });

        // Minecraft wants the favicon key to be omitted instead of `null`.
        // Serde serializes `Option::None` to `null` instead of omitting the key,
        // so we need to resort to mutation here.
        let favicon = self
            .favicon
            .as_ref()
            .map(|i| format!("data:image/png;base64,{}", i));
        if let Some(icon) = favicon {
            json.as_object_mut()
                .expect("wanted object")
                .insert("favicon".to_owned(), Value::String(icon));
        }

        json.to_string()
    }
}

impl From<crate::net::Status> for StatusResponse {
    fn from(stats: crate::net::Status) -> Self {
        StatusResponse {
            version: "1.13.2".to_owned(),
            protocol_version: 404,
            players_max: stats.players_max,
            players_online: stats.players_online,
            description: stats.description,
            favicon: stats.favicon,
        }
    }
}

impl Outgoing for StatusResponse {
    fn written_len(&self) -> usize {
        let stringified = self.build_json();

        var_usize_length(stringified.len()) + stringified.len()
    }

    fn write_to(&self, dst: &mut BytesMut) -> io::Result<()> {
        let stringified = self.build_json();
        dst.write_str(&stringified)
    }
}
