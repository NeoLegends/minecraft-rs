#![feature(async_await)]
#![allow(dead_code)]

use self::{game::GameBuilder, net::ServerBuilder};
use futures::{channel::mpsc, try_join};
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
        .stats_request(status_request_tx)
        .run();

    try_join!(game, network).map(|_| ())
}
