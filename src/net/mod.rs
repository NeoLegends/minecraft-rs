use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::{self, AbortRegistration, Abortable},
    prelude::*,
};
use log::error;
use reqwest::r#async::Client as HttpClient;
use std::{io, net::SocketAddr};
use tokio::net::TcpListener;

#[macro_use]
mod macros;

mod connection;
mod crypto;
mod status_request;
mod util;

pub mod packets;
pub use self::status_request::*;

#[derive(Debug)]
pub struct Client {
    incoming: Receiver<packets::IncomingPackets>,
    outgoing: Sender<packets::OutgoingPackets>,
    username: String,
}

#[derive(Debug)]
pub struct ServerBuilder {
    bind_addr: Option<SocketAddr>,
    new_player: Option<Sender<Client>>,
    shutdown: Option<AbortRegistration>,
    status_request: Option<Sender<StatusRequest>>,
}

#[derive(Clone)]
pub struct ServerState {
    pub http_client: HttpClient,
    pub keypair: crypto::Keypair,
    pub new_client: Sender<Client>,
    pub status_request: Sender<StatusRequest>,
}

impl Client {
    pub fn new(
        username: String,
    ) -> (
        Sender<packets::IncomingPackets>,
        Receiver<packets::OutgoingPackets>,
        Client,
    ) {
        let (inc_tx, inc_rx) = mpsc::channel(0);
        let (out_tx, out_rx) = mpsc::channel(0);

        (
            inc_tx,
            out_rx,
            Client {
                incoming: inc_rx,
                outgoing: out_tx,
                username,
            },
        )
    }

    pub fn incoming(&mut self) -> &mut Receiver<packets::IncomingPackets> {
        &mut self.incoming
    }

    pub fn outgoing(&mut self) -> &mut Sender<packets::OutgoingPackets> {
        &mut self.outgoing
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder {
            bind_addr: None,
            new_player: None,
            shutdown: None,
            status_request: None,
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

    pub fn status_request(mut self, on_request: Sender<StatusRequest>) -> Self {
        self.status_request = Some(on_request);
        self
    }

    pub async fn run(self) -> io::Result<()> {
        let bind_addr = self.bind_addr.expect("missing bind_addr");
        let new_client = self.new_player.expect("missing channel for new players");
        let status_request = self
            .status_request
            .expect("missing channel for status requests");
        let keypair = crypto::Keypair::generate();

        let state = ServerState {
            http_client: HttpClient::new(),
            keypair,
            new_client,
            status_request,
        };

        let handler_fut =
            TcpListener::bind(&bind_addr)?
                .incoming()
                .for_each(|maybe_conn| {
                    match maybe_conn {
                        Ok(conn) => connection::accept(conn, state.clone()),
                        Err(e) => {
                            error!("error while accepting TCP connection: {:?}", e)
                        }
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
