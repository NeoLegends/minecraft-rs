use futures::{
    channel::mpsc::Sender,
    future::{self, AbortRegistration, Abortable},
    prelude::*,
};
use std::{io, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};

#[macro_use]
mod macros;

pub mod crypto;
pub mod packets;
pub mod util;

#[derive(Debug)]
pub struct Client {}

#[derive(Debug)]
pub struct ServerBuilder {
    bind_addr: Option<SocketAddr>,
    new_player: Option<Sender<Client>>,
    shutdown: Option<AbortRegistration>,
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

    pub async fn run(self) -> io::Result<()> {
        let bind_addr = self.bind_addr.expect("missing bind_addr");
        let new_player = self.new_player.expect("missing channel for new players");

        let handler_fut =
            TcpListener::bind(&bind_addr)?
                .incoming()
                .for_each(|maybe_conn| {
                    match maybe_conn {
                        Ok(conn) => {
                            let accept_future =
                                accept_connection(conn, new_player.clone());
                            tokio::spawn(accept_future);
                        }
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

async fn accept_connection(conn: TcpStream, new_player: Sender<Client>) {}
