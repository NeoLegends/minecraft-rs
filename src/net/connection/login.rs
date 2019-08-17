use crate::net::{packets::*, Client, ConnectionState};
use futures::channel::mpsc::Sender;
use std::io;
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    _new_player: Sender<Client>,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Login);

    unimplemented!()
}
