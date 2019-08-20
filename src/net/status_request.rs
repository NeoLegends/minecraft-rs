use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Status {
    pub players_max: usize,
    pub players_online: usize,
    pub description: String,
    pub favicon: Option<String>,
}

#[derive(Debug)]
pub struct StatusRequest {
    send_stats: oneshot::Sender<Status>,
}

impl StatusRequest {
    pub async fn send_via(mut chan: mpsc::Sender<StatusRequest>) -> Option<Status> {
        let (tx, rx) = oneshot::channel();
        let request = StatusRequest { send_stats: tx };

        match chan.send(request).await.ok() {
            Some(_) => rx.map(|res| res.ok()).await,
            None => None,
        }
    }

    pub fn respond(self, stats: Status) {
        let _ = self.send_stats.send(stats);
    }
}
