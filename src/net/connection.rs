use super::{packets::*, Client, ConnectionState, StatsRequest};
use futures::{
    channel::{mpsc::Sender, oneshot},
    prelude::*,
};
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, io::AsyncWriteExt, net::TcpStream};

pub fn accept(
    conn: TcpStream,
    new_player: Sender<Client>,
    stats_request: Sender<StatsRequest>,
) {
    tokio::spawn(async {
        let res = handle_connection(conn, new_player, stats_request).await;

        if let Err(e) = res {
            eprintln!("{:?}", e);
        }
    });
}

async fn handle_connection(
    conn: TcpStream,
    new_player: Sender<Client>,
    stats_request: Sender<StatsRequest>,
) -> io::Result<()> {
    let mut framed = Framed::new(conn, Coder::new(ConnectionState::Start));

    let handshake = framed
        .next()
        .await
        .ok_or_else(|| {
            Error::new(ErrorKind::ConnectionAborted.into(), "connection lost")
        })??
        .into_handshake()
        .map_err(|_| {
            Error::new(ErrorKind::InvalidData, "expected handshake packet")
        })?
        .validate_self()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    match handshake.next_state {
        NextState::Login => handle_login(framed, new_player).await?,
        NextState::Status => handle_status(framed, stats_request).await?,
    }

    Ok(())
}

async fn handle_login(
    _conn: Framed<TcpStream, Coder>,
    _new_player: Sender<Client>,
) -> io::Result<()> {
    unimplemented!()
}

async fn handle_status(
    mut conn: Framed<TcpStream, Coder>,
    mut stats_request: Sender<StatsRequest>,
) -> io::Result<()> {
    match conn.next().await {
        Some(Ok(IncomingPackets::StatusHandshake(pkg)))
            if pkg.validate().is_ok() => {}
        _ => {
            println!("connection lost before status response sent.");
            return Ok(());
        }
    }

    let (tx, rx) = oneshot::channel();
    let stats_req = StatsRequest::new(tx);

    stats_request
        .send(stats_req)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "game disconnected"))?;
    let stats = rx
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "game disconnected"))?
        .into();

    conn.send(OutgoingPackets::StatusResponse(stats)).await?;

    match conn.next().await {
        Some(Ok(IncomingPackets::Ping(ping))) => {
            conn.send(OutgoingPackets::Ping(Ping { value: ping.value }))
                .await?;
        }
        _ => {}
    };

    let mut transport = conn.into_inner();
    let _ = AsyncWriteExt::shutdown(&mut transport).await;

    Ok(())
}