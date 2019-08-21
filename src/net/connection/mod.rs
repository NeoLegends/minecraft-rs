use super::{packets::*, ServerState};
use futures::prelude::*;
use log::{error, info};
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, io::AsyncWriteExt, net::TcpStream};

mod login;
mod play;
mod status;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum ConnectionState {
    Start,
    Status,
    Play,
    Login,
}

pub fn accept(conn: TcpStream, state: ServerState) {
    tokio::spawn(async {
        let res = handle_connection(conn, state).await;

        if let Err(e) = res {
            error!("{:?}", e);
        }
    });
}

async fn handle_connection(conn: TcpStream, state: ServerState) -> io::Result<()> {
    let remote_addr = conn.peer_addr()?;
    info!("accepting connection from {}", remote_addr);

    let mut framed = Framed::new(conn, Coder::new(ConnectionState::Start));

    let handshake =
        if let Some(Ok(IncomingPackets::Handshake(hs))) = framed.next().await {
            hs.validate_self()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        } else {
            error!("invalid handshake from {}", remote_addr);

            let mut transport = framed.into_inner();
            let _ = AsyncWriteExt::shutdown(&mut transport).await;
            return Err(Error::new(ErrorKind::InvalidData, "invalid handshake"));
        };
    let mut transport = match handshake.next_state {
        NextState::Login => login::handle(framed, state).await?,
        NextState::Status => status::handle(framed, state).await?,
    };

    let _ = AsyncWriteExt::shutdown(&mut transport).await;
    info!("connection to {} shut down", remote_addr);

    Ok(())
}
