use std::error::Error;
use std::time::Duration;

use rppal::gpio::{Gpio, InputPin, Level, Trigger};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use ButtonMessage::*;

#[derive(Clone, Debug)]
pub enum ButtonMessage {
    Press,
    Inactivity,
}

pub struct ButtonActor {
    button_debounce_ms: u64,
    sleep_timeout_secs: u64,
    pin: InputPin,
    pin_rx: mpsc::Receiver<Level>,
    tx: broadcast::Sender<ButtonMessage>,
}

#[derive(Clone)]
pub struct ButtonActorHandle {
    tx: broadcast::Sender<ButtonMessage>
}

impl ButtonActor {

    fn new_timer(&self, timeout: u64) -> JoinHandle<()> {
        let tx = self.tx.clone();
        tokio::task::spawn(async move {
            sleep(Duration::from_secs(timeout)).await;
            tx.send(Inactivity).unwrap_or_default();
        })
    }

    async fn run(&mut self) {
        let mut timer = self.new_timer(self.sleep_timeout_secs * 2);
        while let Some(_level) = self.pin_rx.recv().await {
            timer.abort();
            timer = self.new_timer(self.sleep_timeout_secs);
            sleep(Duration::from_millis(self.button_debounce_ms)).await;
            if self.pin.is_high() { continue; } // signal is debounced after this
            self.tx.send(Press).unwrap_or_default();
        }
    }

    pub fn new(button_bcm_pin: u8, button_debounce_ms: u64, sleep_timeout_secs: u64) -> Result<ButtonActorHandle, Box<dyn Error>> {
        // setup pin and callback
        let gpio = Gpio::new()?;
        let mut pin = gpio.get(button_bcm_pin)?.into_input_pullup();
        let (pin_tx, pin_rx) = mpsc::channel(1);
        pin.set_async_interrupt(Trigger::FallingEdge, move |level| pin_tx.blocking_send(level).unwrap_or_default())?;
        // listen to pin state changes
        let (tx, _) = broadcast::channel(1);
        let tx2 = tx.clone();
        let mut actor = ButtonActor { button_debounce_ms, sleep_timeout_secs,  pin, pin_rx, tx };
        tokio::task::spawn(async move { actor.run().await });
        Ok(ButtonActorHandle { tx: tx2 })
    }

}

impl ButtonActorHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<ButtonMessage> {
        self.tx.subscribe()
    }
}
