use std::error::Error;
use std::iter::{Cycle, Skip};
use std::vec::IntoIter;

use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use HmiMessage::*;

use crate::actor::button::{ButtonActor, ButtonMessage};
use crate::actor::datalogger::{DataLoggerHandle, DataLoggerMessage};
use crate::actor::display::{DisplayActor, DisplayActorHandle};
use crate::actor::linky::{LinkyActorHandle, LinkyMessage};
use crate::actor::rpict::{RpictActorHandle, RpictMessage};
use crate::display::pages::{StartupPage, LinkyPage, Page, RpictPage};
use crate::settings;

type Carrousel = Skip<Cycle<IntoIter<Page>>>;

#[derive(Debug)]
enum HmiMessage {
    Shutdown(oneshot::Sender<()>),
}

pub struct HmiActor {
    // actor refs
    rpict_rx: broadcast::Receiver<RpictMessage>,
    linky_rx: broadcast::Receiver<LinkyMessage>,
    button_rx: broadcast::Receiver<ButtonMessage>,
    datalogger_rx: broadcast::Receiver<DataLoggerMessage>,
    rx: mpsc::Receiver<HmiMessage>,
    display: DisplayActorHandle,
    // internal state
    startup_page: StartupPage,
    rpict_page: RpictPage,
    linky_page: LinkyPage,
    carousel: Carrousel,
    is_display_active: bool,
}

#[derive(Clone)]
pub struct HmiActorHandle {
    tx: mpsc::Sender<HmiMessage>
}

impl HmiActor {

    async fn handle_rpict(&mut self, msg: RpictMessage) {
        match msg {
            RpictMessage::Connected => {
                log::info!("Rpict connected");
                self.startup_page.rpict_status(true);
                self.display.display_startup_page(&self.startup_page, false).await;
            },
            RpictMessage::Disconnected => {
                log::warn!("Rpict disconnected");
                self.startup_page.rpict_status(false);
                self.display.display_startup_page(&self.startup_page, false).await;
            },
            RpictMessage::NewFrame(frame) => {
                log::trace!("New Rpict frame: {:?}", frame);
                self.rpict_page.update(
                    frame.l1_apparent_power,
                    frame.l2_apparent_power,
                    frame.l3_apparent_power,
                    frame.l1_vrms,
                    frame.l2_vrms,
                    frame.l3_vrms
                );
                self.display.display_rpict_page(&self.rpict_page, false).await;
            },
        }
    }

    async fn handle_linky(&mut self, msg: LinkyMessage) {
        match msg {
            LinkyMessage::Connected => {
                log::info!("Linky connected");
                self.startup_page.linky_status(true);
                self.display.display_startup_page(&self.startup_page, false).await;
            },
            LinkyMessage::Disconnected => {
                log::warn!("Linky disconnected");
                self.startup_page.linky_status(false);
                self.display.display_startup_page(&self.startup_page, false).await;
            },
            LinkyMessage::NewFrame(frame) => {
                log::trace!("New Linky frame: {:?}", frame);
                self.linky_page.update(
                    frame.adco.clone(),
                    frame.hchp,
                    frame.hchc,
                    frame.ptec()
                );
                self.display.display_linky_page(&self.linky_page, false).await;
            },
        }
    }

    async fn handle_datalogger(&mut self, msg: DataLoggerMessage) {
        match msg {
            DataLoggerMessage::Connected => {
                log::info!("Data logger connected");
                self.startup_page.influxdb_status(true);
                self.display.display_startup_page(&self.startup_page, false).await;
            },
            DataLoggerMessage::Disconnected => {
                log::warn!("Data logger disconnected");
                self.startup_page.influxdb_status(false);
                self.display.display_startup_page(&self.startup_page, false).await;
            }
        }
    }

    async fn handle_button(&mut self, msg: ButtonMessage) {
        match msg {
            ButtonMessage::Press => {
                log::debug!("Button press");
                if self.is_display_active {
                    match self.carousel.next().unwrap() {
                        Page::Startup => self.display.display_startup_page(&self.startup_page, true).await,
                        Page::Rpict => self.display.display_rpict_page(&self.rpict_page, true).await,
                        Page::Linky => self.display.display_linky_page(&self.linky_page, true).await,
                    }
                } else {
                    self.is_display_active = true;
                    self.display.set_display_on().await;
                }
            },
            ButtonMessage::Inactivity => {
                log::debug!("Button inactivity");
                self.is_display_active = false;
                self.display.set_display_off().await;
            },
        }
    }

    async fn run(&mut self) {
        self.display.display_startup_page(&self.startup_page, true).await;
        self.display.set_display_on().await;
        loop {
            tokio::select! {
                Ok(msg) = self.rpict_rx.recv() => self.handle_rpict(msg).await,
                Ok(msg) = self.linky_rx.recv() => self.handle_linky(msg).await,
                Ok(msg) = self.datalogger_rx.recv() => self.handle_datalogger(msg).await,
                Ok(msg) = self.button_rx.recv() => self.handle_button(msg).await,
                Some(msg) = self.rx.recv() => match msg {
                    HmiMessage::Shutdown(callback) => {
                        log::debug!("Shutdown hmi");
                        self.display.shutdown().await;
                        callback.send(()).unwrap_or_default();
                        break;
                    },
                },
                else => break,
            }
        }
    }

    pub fn create(
        settings: &settings::Hmi,
        rpict: &RpictActorHandle,
        linky: &LinkyActorHandle,
        datalogger: &DataLoggerHandle,
    ) -> Result<HmiActorHandle, Box<dyn Error>>
    {
        let settings = settings.clone();
        let rpict_rx = rpict.subscribe();
        let linky_rx = linky.subscribe();
        let datalogger_rx = datalogger.subscribe();
        // child actors
        let button_rx = ButtonActor::create(
            settings.button_bcm_pin,
            settings.button_debounce_ms,
            settings.sleep_timeout_secs
        )?.subscribe();
        let display = DisplayActor::create()?;
        // pages declaration
        let startup_page = StartupPage::new(env!("CARGO_PKG_VERSION"));
        let rpict_page = RpictPage::new(settings.max_line_power_watts);
        let linky_page = LinkyPage::new();
        let carousel: Carrousel = vec![Page::Startup, Page::Rpict, Page::Linky].into_iter().cycle().skip(1);
        // fork
        let (tx, rx) = mpsc::channel(1);
        let mut actor = HmiActor {
            rpict_rx, linky_rx, button_rx, datalogger_rx, rx, display,
            startup_page, rpict_page, linky_page, carousel, is_display_active: true,
        };
        tokio::task::spawn(async move { actor.run().await });
        Ok(HmiActorHandle { tx })
    }

}

impl HmiActorHandle {
    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Shutdown(tx)).await.unwrap_or_default();
        rx.await.unwrap_or_default()
    }
}
