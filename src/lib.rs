#![allow(dead_code)]

use self::{game::GameBuilder, net::ServerBuilder};
use futures::{channel::mpsc, prelude::*, try_join};
use std::{io, path::Path};

mod game;
mod net;

pub async fn run(world: &Path, port: u16) -> io::Result<()> {
    let (new_player_tx, new_player_rx) = mpsc::channel(0);
    let (status_request_tx, status_request_rx) = mpsc::channel(0);
    let bind_addr = ([0, 0, 0, 0], port).into();

    let game = GameBuilder::new()
        .new_players(new_player_rx)
        .status_requests(status_request_rx)
        .world(world)
        .run();
    let network = ServerBuilder::new()
        .bind_addr(bind_addr)
        .new_player(new_player_tx)
        .status_request(status_request_tx)
        .run();

    try_join!(game, network).map(|_| ())
}

pub async fn run_test() {
    let (new_player_tx, _) = mpsc::channel(0);
    let (status_request_tx, mut status_request_rx) = mpsc::channel(0);

    let network = ServerBuilder::new()
        .bind_addr(([127, 0, 0, 1], 25565).into())
        .new_player(new_player_tx)
        .status_request(status_request_tx)
        .run();

    tokio::spawn(async move {
        while let Some(x) = status_request_rx.next().await {
            let stats = net::Status {
                players_max: 100,
                players_online: 0,
                description: "Bla bla bla".to_owned(),
                favicon: None,
            };

            x.respond(stats);
        }
    });

    let _ = network.await;
}
