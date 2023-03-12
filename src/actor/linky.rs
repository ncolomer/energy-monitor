use std::thread::sleep;
use std::time::Duration;
use tokio::sync::broadcast;

use LinkyMessage::*;

use crate::driver::linky::{Linky, LinkyFrame};

#[derive(Clone, Debug)]
pub enum LinkyMessage {
    Connected,
    Disconnected,
    NewFrame(LinkyFrame),
}

pub struct LinkyActor;

#[derive(Clone)]
pub struct LinkyActorHandle {
    tx: broadcast::Sender<LinkyMessage>
}

impl LinkyActor {

    pub fn new(serial_path: &String) -> LinkyActorHandle {
        let serial_path = serial_path.clone();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            let iter = Linky::new()
                .with_port_path(serial_path)
                .bind();
            if let Err(_) = iter {
                log::warn!("Cannot connect Linky");
                tx.send(Disconnected).unwrap_or_default();
                return;
            }
            sleep(Duration::from_secs(1));
            tx.send(Connected).unwrap_or_default();
            for frame in iter.unwrap() {
                if let Err(_) = tx.send(NewFrame(frame)) { break; }
            }
        });
        LinkyActorHandle { tx: tx2 }
    }

}

impl LinkyActorHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<LinkyMessage> {
        self.tx.subscribe()
    }
}
