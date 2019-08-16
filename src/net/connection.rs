use super::{packets::*, Client, ConnectionState, StatsRequest};
use futures::{channel::mpsc::Sender, prelude::*};
use log::{error, info};
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
            error!("{:?}", e);
        }
    });
}

async fn handle_connection(
    conn: TcpStream,
    new_player: Sender<Client>,
    stats_request: Sender<StatsRequest>,
) -> io::Result<()> {
    let mut framed = Framed::new(conn, Coder::new(ConnectionState::Start));

    let maybe_handshake = framed
        .next()
        .await
        .ok_or_else(|| Error::new(ErrorKind::ConnectionAborted, "connection lost"))
        .and_then(|inner| inner)
        .and_then(|packet| {
            packet.into_handshake().map_err(|_| {
                Error::new(ErrorKind::InvalidData, "expected handshake packet")
            })
        })
        .and_then(|hs| {
            hs.validate_self()
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        });
    let handshake = match maybe_handshake {
        Ok(hs) => hs,
        Err(e) => {
            let mut transport = framed.into_inner();
            let _ = AsyncWriteExt::shutdown(&mut transport).await;
            return Err(e);
        }
    };

    match handshake.next_state {
        NextState::Login => handle_login(framed, new_player).await?,
        NextState::Status => handle_status(framed, stats_request).await?,
    }

    Ok(())
}

async fn handle_login(
    mut conn: Framed<TcpStream, Coder>,
    _new_player: Sender<Client>,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Login);

    unimplemented!()
}

async fn handle_status(
    mut conn: Framed<TcpStream, Coder>,
    mut stats_request: Sender<StatsRequest>,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Status);

    match conn.next().await {
        Some(Ok(IncomingPackets::StatusHandshake(pkg)))
            if pkg.validate().is_ok() => {}
        _ => {
            info!("connection lost before status response sent.");
            return Ok(());
        }
    }

    let (stats_req, stats_resp) = StatsRequest::new();
    stats_request
        .send(stats_req)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "game disconnected"))?;
    let stats = stats_resp
        .await
        .ok_or_else(|| Error::new(ErrorKind::Other, "game disconnected"))?
        .into();

    conn.send(OutgoingPackets::StatusResponse(stats)).await?;

    if let Some(Ok(IncomingPackets::Ping(ping))) = conn.next().await {
        conn.send(OutgoingPackets::Ping(Ping { value: ping.value }))
            .await?;
    }

    let mut transport = conn.into_inner();
    let _ = AsyncWriteExt::shutdown(&mut transport).await;

    Ok(())
}
