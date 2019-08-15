use futures::{
    channel::{mpsc::Sender, oneshot},
    future::{self, AbortRegistration, Abortable},
    prelude::*,
};
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;

#[macro_use]
mod macros;

mod connection;
mod crypto;
mod util;

pub mod packets;

#[derive(Debug)]
pub struct Client {}

#[derive(Debug)]
pub struct ServerBuilder {
    bind_addr: Option<SocketAddr>,
    new_player: Option<Sender<Client>>,
    shutdown: Option<AbortRegistration>,
    stats_request: Option<Sender<StatsRequest>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Stats {
    pub players_max: usize,
    pub players_online: usize,
    pub description: String,
    pub favicon: Option<String>,
}

#[derive(Debug)]
pub struct StatsRequest {
    send_stats: oneshot::Sender<Stats>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum ConnectionState {
    Start,
    Status,
    Play,
    Login,
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder {
            bind_addr: None,
            new_player: None,
            shutdown: None,
            stats_request: None,
        }
    }

    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = Some(addr);
        self
    }

    pub fn new_player(mut self, new_player: Sender<Client>) -> Self {
        self.new_player = Some(new_player);
        self
    }

    pub fn shutdown_on(mut self, reg: AbortRegistration) -> Self {
        self.shutdown = Some(reg);
        self
    }

    pub fn stats_request(mut self, on_request: Sender<StatsRequest>) -> Self {
        self.stats_request = Some(on_request);
        self
    }

    pub async fn run(self) -> io::Result<()> {
        let bind_addr = self.bind_addr.expect("missing bind_addr");
        let new_player = self.new_player.expect("missing channel for new players");
        let stats_req = self
            .stats_request
            .expect("missing channel for status requests");

        let handler_fut =
            TcpListener::bind(&bind_addr)?
                .incoming()
                .for_each(|maybe_conn| {
                    match maybe_conn {
                        Ok(conn) => connection::accept(
                            conn,
                            new_player.clone(),
                            stats_req.clone(),
                        ),
                        Err(e) => eprintln!(
                            "error while accepting TCP connection: {:?}",
                            e
                        ),
                    }

                    future::ready(())
                });

        if let Some(shutdown) = self.shutdown {
            let _ = Abortable::new(handler_fut, shutdown).await;
        } else {
            handler_fut.await
        }

        Ok(())
    }
}

impl StatsRequest {
    pub fn new(send_stats: oneshot::Sender<Stats>) -> Self {
        StatsRequest { send_stats }
    }

    pub fn respond_to(self) -> oneshot::Sender<Stats> {
        self.send_stats
    }
}
