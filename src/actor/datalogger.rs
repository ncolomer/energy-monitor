use std::error::Error;

use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use DataLoggerMessage::*;

use crate::actor::linky::{LinkyActorHandle, LinkyMessage};
use crate::actor::rpict::{RpictActorHandle, RpictMessage};

use crate::service::influxdb::InfluxDBClient;
use crate::settings;

#[derive(Clone, Debug)]
pub enum DataLoggerMessage {
    InfluxDbConnected,
    InfluxDbDisconnected,
}

pub struct DataLoggerActor {
    influxdb: Option<InfluxDBClient>,
    rpict_rx: broadcast::Receiver<RpictMessage>,
    linky_rx: broadcast::Receiver<LinkyMessage>,
    tx: broadcast::Sender<DataLoggerMessage>,
}

#[derive(Clone)]
pub struct DataLoggerHandle {
    tx: broadcast::Sender<DataLoggerMessage>,
}

impl DataLoggerActor {
    async fn run(&mut self) {
        let mut influxdb_connected = false;
        loop {
            tokio::select! {
                msg = self.rpict_rx.recv() => match msg {
                    Ok(RpictMessage::NewFrame(frame)) => {
                        log::trace!("New Rpict frame: {:?}", frame);
                        if let Some(client) = &self.influxdb {
                            if client.publish(&frame).await.is_ok() {
                                if !influxdb_connected {
                                    self.tx.send(InfluxDbConnected).unwrap_or_default();
                                    influxdb_connected = true;
                                }
                            } else if influxdb_connected {
                                self.tx.send(InfluxDbDisconnected).unwrap_or_default();
                                influxdb_connected = false;
                            }
                        }
                    },
                    Err(RecvError::Lagged(skipped)) => {
                        log::warn!("Lag while logging rpict data, skipped {:?} frames", skipped);
                    },
                    _ => {}
                },
                msg = self.linky_rx.recv() => match msg {
                    Ok(LinkyMessage::NewFrame(frame)) => {
                        log::trace!("New Linky frame: {:?}", frame);
                        if let Some(client) = &self.influxdb {
                            if client.publish(&frame).await.is_ok() {
                                if !influxdb_connected {
                                    self.tx.send(InfluxDbConnected).unwrap_or_default();
                                    influxdb_connected = true;
                                }
                            } else if influxdb_connected {
                                self.tx.send(InfluxDbDisconnected).unwrap_or_default();
                                influxdb_connected = false;
                            }
                        }
                    },
                    Err(RecvError::Lagged(skipped)) => {
                        log::warn!("Lag while logging linky data, skipped {:?} frames", skipped);
                    },
                    _ => {}
                },
                else => break,
            }
        }
    }

    pub fn create(
        influxdb_settings: &Option<settings::InfluxDB>,
        rpict: &RpictActorHandle,
        linky: &LinkyActorHandle,
    ) -> Result<DataLoggerHandle, Box<dyn Error>> {
        let influxdb = influxdb_settings.map(|settings| InfluxDBClient::new(&settings).unwrap());
        let rpict_rx = rpict.subscribe();
        let linky_rx = linky.subscribe();
        // fork
        let (tx, _) = broadcast::channel(1);
        let mut actor = DataLoggerActor {
            influxdb,
            rpict_rx,
            linky_rx,
            tx: tx.clone(),
        };
        tokio::task::spawn(async move { actor.run().await });
        Ok(DataLoggerHandle { tx })
    }
}

impl DataLoggerHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<DataLoggerMessage> {
        self.tx.subscribe()
    }
}
