use crate::net::{Client, StatusRequest};
use futures::channel::mpsc::Receiver;
use std::{io, path::Path};

mod world;

#[derive(Debug)]
pub struct GameBuilder<'a> {
    new_players: Option<Receiver<Client>>,
    status_requests: Option<Receiver<StatusRequest>>,
    world: Option<&'a Path>,
}

impl<'a> GameBuilder<'a> {
    pub fn new() -> Self {
        GameBuilder {
            new_players: None,
            status_requests: None,
            world: None,
        }
    }

    pub fn new_players(mut self, recv: Receiver<Client>) -> Self {
        self.new_players = Some(recv);
        self
    }

    pub fn status_requests(mut self, recv: Receiver<StatusRequest>) -> Self {
        self.status_requests = Some(recv);
        self
    }

    pub fn world(mut self, path: &'a Path) -> Self {
        self.world = Some(path);
        self
    }

    pub async fn run(self) -> io::Result<()> {
        let _ = self.new_players.expect("missing new players receiver");
        let _ = self
            .status_requests
            .expect("missing status requests receiver");
        let _ = self.world.expect("missing world path");

        Ok(())
    }
}
