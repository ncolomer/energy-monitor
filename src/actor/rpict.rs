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

    pub fn new(serial_path: &String) -> RpictActorHandle {
        let serial_path = serial_path.clone();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            let iter = Rpict::new()
                .with_port_path(serial_path)
                .bind();
            if let Err(_) = iter {
                log::warn!("Cannot connect Rpict");
                tx.send(Disconnected).unwrap_or_default();
                return;
            }
            sleep(Duration::from_secs(1));
            tx.send(Connected).unwrap_or_default();
            for frame in iter.unwrap() {
                if let Err(_) = tx.send(NewFrame(frame)) { break; }
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
