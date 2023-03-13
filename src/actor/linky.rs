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

    pub fn create(serial_path: &str) -> LinkyActorHandle {
        let serial_path = serial_path.to_owned();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            sleep(Duration::from_secs(1));
            let iter = Linky::builder()
                .with_port_path(serial_path)
                .build();
            if let Err(e) = iter {
                log::debug!("Cannot connect Linky: {:?}", e);
                tx.send(Disconnected).unwrap_or_default();
                return;
            } else {
                tx.send(Connected).unwrap_or_default();
            }
            for frame in iter.unwrap() {
                if tx.send(NewFrame(frame)).is_err() { break; }
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
