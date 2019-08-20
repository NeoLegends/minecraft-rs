use super::ConnectionState;
use crate::net::{packets::*, ServerState};
use futures::prelude::*;
use log::error;
use rand;
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    state: ServerState,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Login);

    let username = match conn.next().await {
        Some(Ok(IncomingPackets::LoginStart(packet))) => {
            packet
                .validate_self()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
                .username
        }
        _ => {
            error!("invalid login handshake");
            return Ok(conn.into_inner());
        }
    };

    let verify_token = rand::random();
    let enc_request = EncryptionRequest {
        server_id: String::new(),
        public_key: state.keypair.public,
        verify_token,
    };
    conn.send(OutgoingPackets::EncryptionRequest(enc_request))
        .await?;

    unimplemented!()
}
