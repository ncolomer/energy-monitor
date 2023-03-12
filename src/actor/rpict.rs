use std::thread::sleep;
use std::time::Duration;
use tokio::sync::broadcast;

use RpictMessage::*;

use crate::driver::rpict::{Rpict, RpictFrame};

#[derive(Clone, Debug)]
pub enum RpictMessage {
    Connected,
    Disconnected,
    NewFrame(RpictFrame),
}

pub struct RpictActor;

#[derive(Clone)]
pub struct RpictActorHandle {
    tx: broadcast::Sender<RpictMessage>
}

impl RpictActor {

    pub fn create(serial_path: &str) -> RpictActorHandle {
        let serial_path = serial_path.to_owned();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            let iter = Rpict::builder()
                .with_port_path(serial_path)
                .build();
            if iter.is_err() {
                log::warn!("Cannot connect Rpict");
                tx.send(Disconnected).unwrap_or_default();
                return;
            }
            sleep(Duration::from_secs(1));
            tx.send(Connected).unwrap_or_default();
            for frame in iter.unwrap() {
                if tx.send(NewFrame(frame)).is_err() { break; }
            }
        });
        RpictActorHandle { tx: tx2 }
    }

}

impl RpictActorHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<RpictMessage> {
        self.tx.subscribe()
    }
}
