use super::ConnectionState;
use crate::net::{crypto::CryptStream, packets::*, util::Autoflush, ServerState};
use futures::channel::mpsc;
use std::io;
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<Autoflush<CryptStream<TcpStream>>, Coder>,
    _state: ServerState,
    _incoming: mpsc::Sender<IncomingPackets>,
    _outgoing: mpsc::Receiver<OutgoingPackets>,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Play);

    unimplemented!()
}
