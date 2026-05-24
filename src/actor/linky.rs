use std::thread::sleep;
use tokio::sync::broadcast;

use LinkyMessage::*;

use crate::actor::blocking::{run_blocking_actor, ConnectionMessage};
use crate::driver::linky::{Linky, LinkyFrame};

#[derive(Clone, Debug)]
pub enum LinkyMessage {
    Connected,
    Disconnected,
    NewFrame(LinkyFrame),
}

impl ConnectionMessage for LinkyMessage {
    type Frame = LinkyFrame;
    fn connected() -> Self {
        Connected
    }
    fn disconnected() -> Self {
        Disconnected
    }
    fn frame(frame: LinkyFrame) -> Self {
        NewFrame(frame)
    }
}

pub struct LinkyActor;

#[derive(Clone)]
pub struct LinkyActorHandle {
    tx: broadcast::Sender<LinkyMessage>,
}

impl LinkyActor {
    pub fn create(serial_path: &str) -> LinkyActorHandle {
        let serial_path = serial_path.to_owned();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            let attempts = std::iter::repeat_with(move || {
                Linky::builder().with_port_path(serial_path.clone()).build()
            });
            run_blocking_actor::<LinkyMessage, _, _, _>(attempts, tx, sleep);
        });
        LinkyActorHandle { tx: tx2 }
    }
}

impl LinkyActorHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<LinkyMessage> {
        self.tx.subscribe()
    }
}
