use super::ConnectionState;
use crate::net::{packets::*, ServerState, StatusRequest};
use futures::prelude::*;
use log::info;
use std::io::{self, Error, ErrorKind};
use tokio::{codec::Framed, net::TcpStream};

pub async fn handle(
    mut conn: Framed<TcpStream, Coder>,
    state: ServerState,
) -> io::Result<TcpStream> {
    conn.codec_mut().set_state(ConnectionState::Status);

    if let Some(Ok(IncomingPackets::StatusHandshake(hs))) = conn.next().await {
        hs.validate()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    } else {
        info!("connection lost before status response sent.");
        return Ok(conn.into_inner());
    }

    let stats = StatusRequest::send_via(state.stats_request)
        .await
        .ok_or_else(|| Error::new(ErrorKind::Other, "game disconnected"))?;
    conn.send(OutgoingPackets::StatusResponse(stats.into()))
        .await?;

    while let Some(Ok(IncomingPackets::Ping(ping))) = conn.next().await {
        conn.send(OutgoingPackets::Ping(Ping { value: ping.value }))
            .await?;
    }

    Ok(conn.into_inner())
}
