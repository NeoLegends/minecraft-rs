use super::{play, ConnectionState};
use crate::net::{
    crypto::{self, CryptStream},
    packets::*,
    util::Autoflush,
    Client, ServerState,
};
use futures::{compat::Future01CompatExt, prelude::*};
use log::error;
use rand;
use reqwest::r#async::Client as HttpClient;
use serde::Deserialize;
use std::io::{self, Error, ErrorKind};
use tokio::{
    codec::{Framed, FramedParts},
    net::TcpStream,
};

const SESSION_SERVER_URL: &str =
    "https://sessionserver.mojang.com/session/minecraft/hasJoined";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct ClientValidation {
    id: String,
    name: String,
    properties: Vec<Property>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Property {
    name: String,
    value: String,
    signature: String,
}

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    mut state: ServerState,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Login);

    let username = expect_packet!(conn, LoginStart).username;

    let verify_token = rand::random();
    let enc_request = EncryptionRequest {
        server_id: String::new(),
        public_key: state.keypair.public.clone(),
        verify_token,
    };
    conn.send(OutgoingPackets::EncryptionRequest(enc_request))
        .await?;

    let decrypted = expect_packet!(conn, EncryptionResponse)
        .decrypt_parts(&state.keypair.private)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    if decrypted.verify_token != verify_token {
        error!("invalid verify token");
        return Ok(conn.into_inner());
    }

    let login_digest =
        crypto::server_digest(&state.keypair.public, &decrypted.shared_secret);
    let validation =
        ClientValidation::fetch(&state.http_client, &username, &login_digest)
            .await?;

    if validation.username() != username {
        error!("invalid username");
        return Ok(conn.into_inner());
    }

    let login_success = LoginSuccess {
        uuid: validation.id_for_client(),
        username: username.clone(),
    };

    let mut encrypted_conn = {
        let parts = conn.into_parts();

        let mut parts_enc = FramedParts::new(
            Autoflush::new(CryptStream::new(
                parts.io,
                &decrypted.shared_secret,
                &decrypted.shared_secret,
            )),
            parts.codec,
        );
        parts_enc.read_buf = parts.read_buf;
        parts_enc.write_buf = parts.write_buf;

        Framed::from_parts(parts_enc)
    };
    encrypted_conn
        .send(OutgoingPackets::LoginSuccess(login_success))
        .await?;

    let (inc_tx, out_rx, client) = Client::new(username);
    state
        .new_client
        .send(client)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "game disconnected"))?;

    play::handle(encrypted_conn, state, inc_tx, out_rx).await
}

impl ClientValidation {
    pub async fn fetch(
        via: &HttpClient,
        username: &str,
        digest: &str,
    ) -> Result<Self, Error> {
        let mut response = via
            .get(SESSION_SERVER_URL)
            .query(&[("username", username), ("serverId", digest)])
            .send()
            .compat()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|e| Error::new(ErrorKind::Other, e))?;

        response
            .json::<ClientValidation>()
            .compat()
            .await
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn id_for_client(&self) -> String {
        assert_eq!(self.id.len(), 32);

        format!(
            "{}-{}-{}-{}-{}",
            &self.id[..8],
            &self.id[8..12],
            &self.id[12..16],
            &self.id[16..20],
            &self.id[20..],
        )
    }

    pub fn username(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_response_id_for_client() {
        let resp = ClientValidation {
            id: "11111111222233334444555555555555".to_owned(),
            name: String::new(),
            properties: Vec::new(),
        };

        assert_eq!(resp.id_for_client(), "11111111-2222-3333-4444-555555555555");
    }
}
