use super::ConnectionState;
use crate::net::{crypto, packets::*, ServerState};
use futures::prelude::*;
use rand;
use std::io;
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    state: ServerState,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Login);

    let username = expect_packet!(conn, LoginStart).username;

    let verify_token = rand::random();
    let enc_request = EncryptionRequest {
        server_id: String::new(),
        public_key: state.keypair.public,
        verify_token,
    };
    conn.send(OutgoingPackets::EncryptionRequest(enc_request))
        .await?;

    let encryption_response = expect_packet!(conn, EncryptionResponse);

    unimplemented!()
}
