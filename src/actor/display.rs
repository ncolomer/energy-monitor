use std::error::Error;

use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::BinaryColor;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use DisplayMessage::*;

use crate::display::pages::{LandingPage, LinkyPage, Page, RpictPage};
use crate::driver::ssd1305::Ssd1305;

#[derive(Debug)]
pub enum DisplayMessage {
    SetDisplayOn,
    SetDisplayOff,
    DisplayLandingPage { page: LandingPage, replace: bool },
    DisplayRpictPage { page: RpictPage, replace: bool },
    DisplayLinkyPage { page: LinkyPage, replace: bool },
    Shutdown(oneshot::Sender<()>),
}

pub struct DisplayActor {
    rx: mpsc::Receiver<DisplayMessage>,
    driver: Ssd1305,
    current_page: Page,
}

#[derive(Clone)]
pub struct DisplayActorHandle {
    tx: mpsc::Sender<DisplayMessage>,
}

impl DisplayActor {

    pub fn new() -> Result<DisplayActorHandle, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel(1);
        let driver = Ssd1305::new()?;
        let mut actor = DisplayActor { rx, driver, current_page: Page::Landing };
        tokio::task::spawn_blocking(move || actor.run());
        Ok(DisplayActorHandle { tx })
    }

    fn run(&mut self) {
        self.driver.begin().unwrap();
        while let Some(msg) = self.rx.blocking_recv() {
            match msg {
                SetDisplayOn => {
                    log::debug!("Display on");
                    self.driver.display_on().unwrap();
                },
                SetDisplayOff => {
                    log::debug!("Display off");
                    self.driver.display_off().unwrap();
                },
                DisplayLandingPage { page, replace } => {
                    self.update_display(Page::Landing, page, replace);
                }
                DisplayRpictPage { page, replace } => {
                    self.update_display(Page::Rpict, page, replace);
                },
                DisplayLinkyPage { page, replace } => {
                    self.update_display(Page::Linky, page, replace);
                },
                Shutdown(callback) => {
                    log::debug!("Shutdown display");
                    self.driver.display_off().unwrap();
                    self.driver.clear().unwrap();
                    callback.send(()).unwrap_or_default();
                    return;
                }
            }
        }
    }

    fn update_display(&mut self, page: Page, drawable: impl Drawable<Color=BinaryColor>, replace: bool) {
        if replace { self.current_page = page; }
        if self.current_page == page {
            log::debug!("Update display with {:?} page", self.current_page);
            drawable.draw(&mut self.driver).unwrap();
            self.driver.flush().unwrap();
        }
    }
}

impl DisplayActorHandle {

    pub async fn set_display_off(&self) {
        let message = SetDisplayOff;
        self.tx.send(message).await.unwrap_or_default();
    }

    pub async fn set_display_on(&self) {
        let message = SetDisplayOn;
        self.tx.send(message).await.unwrap_or_default();
    }

    pub async fn display_landing_page(&self, page: &LandingPage, replace: bool) {
        let message = DisplayLandingPage { page: page.clone(), replace };
        self.tx.send(message).await.unwrap_or_default();
    }

    pub async fn display_rpict_page(&self, page: &RpictPage, replace: bool) {
        let message = DisplayRpictPage { page: page.clone(), replace };
        self.tx.send(message).await.unwrap_or_default();
    }

    pub async fn display_linky_page(&self, page: &LinkyPage, replace: bool) {
        let message = DisplayLinkyPage { page: page.clone(), replace };
        self.tx.send(message).await.unwrap_or_default();
    }

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Shutdown(tx)).await.unwrap_or_default();
        rx.await.unwrap_or_default()
    }

}
