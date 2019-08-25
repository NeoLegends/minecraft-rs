use super::Incoming;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Ping {
    pub value: i64,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
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

impl Incoming for Ping {}

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

impl Serialize for StatusResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = self.build_json();
        serializer.serialize_str(&string)
    }
}
