use super::ConnectionState;
use crate::net::{crypto::CryptStream, packets::*, util::Autoflush, ServerState};
use futures::prelude::*;
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<Autoflush<CryptStream<TcpStream>>, Coder>,
    state: ServerState,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Play);

    unimplemented!()
}
