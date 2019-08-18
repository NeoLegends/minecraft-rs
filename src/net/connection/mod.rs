use super::{crypto::Keypair, packets::*, Client, ConnectionState, StatsRequest};
use futures::{channel::mpsc::Sender, prelude::*};
use log::{error, info};
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, io::AsyncWriteExt, net::TcpStream};

mod login;
mod status;

pub fn accept(
    conn: TcpStream,
    new_player: Sender<Client>,
    stats_request: Sender<StatsRequest>,
    keypair: Keypair,
) {
    tokio::spawn(async {
        let res = handle_connection(conn, new_player, stats_request, keypair).await;

        if let Err(e) = res {
            error!("{:?}", e);
        }
    });
}

async fn handle_connection(
    conn: TcpStream,
    new_player: Sender<Client>,
    stats_request: Sender<StatsRequest>,
    keypair: Keypair,
) -> io::Result<()> {
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
        NextState::Login => login::handle(framed, new_player, keypair).await?,
        NextState::Status => status::handle(framed, stats_request).await?,
    };

    let _ = AsyncWriteExt::shutdown(&mut transport).await;
    info!("connection to {} shut down", remote_addr);

    Ok(())
}
