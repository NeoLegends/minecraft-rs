use crate::net::{packets::*, ConnectionState, StatsRequest};
use futures::{channel::mpsc::Sender, prelude::*};
use log::info;
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    mut stats_request: Sender<StatsRequest>,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Status);

    match conn.next().await {
        Some(Ok(IncomingPackets::StatusHandshake(pkg)))
            if pkg.validate().is_ok() => {}
        _ => {
            info!("connection lost before status response sent.");
            return Ok(conn.into_inner());
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

    while let Some(Ok(IncomingPackets::Ping(ping))) = conn.next().await {
        conn.send(OutgoingPackets::Ping(Ping { value: ping.value }))
            .await?;
    }

    Ok(conn.into_inner())
}
