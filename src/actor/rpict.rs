use std::thread::sleep;
use tokio::sync::broadcast;

use RpictMessage::*;

use crate::actor::blocking::{run_blocking_actor, ConnectionMessage};
use crate::driver::rpict::{Rpict, RpictFrame};

#[derive(Clone, Debug)]
pub enum RpictMessage {
    Connected,
    Disconnected,
    NewFrame(RpictFrame),
}

impl ConnectionMessage for RpictMessage {
    type Frame = RpictFrame;
    fn connected() -> Self {
        Connected
    }
    fn disconnected() -> Self {
        Disconnected
    }
    fn frame(frame: RpictFrame) -> Self {
        NewFrame(frame)
    }
}

pub struct RpictActor;

#[derive(Clone)]
pub struct RpictActorHandle {
    tx: broadcast::Sender<RpictMessage>,
}

impl RpictActor {
    pub fn create(serial_path: &str) -> RpictActorHandle {
        let serial_path = serial_path.to_owned();
        let (tx, _) = broadcast::channel(5);
        let tx2 = tx.clone();
        tokio::task::spawn_blocking(move || {
            let attempts = std::iter::repeat_with(move || {
                Rpict::builder().with_port_path(serial_path.clone()).build()
            });
            run_blocking_actor::<RpictMessage, _, _, _>(attempts, tx, sleep);
        });
        RpictActorHandle { tx: tx2 }
    }
}

impl RpictActorHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<RpictMessage> {
        self.tx.subscribe()
    }
}
