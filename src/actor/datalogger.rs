use std::error::Error;

use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use DataLoggerMessage::*;

use crate::actor::linky::{LinkyActorHandle, LinkyMessage};
use crate::actor::rpict::{RpictActorHandle, RpictMessage};
use crate::service::hassmqtt::{HassMqttClient, ToHassMqtt};
use crate::service::influxdb::{InfluxDBClient, ToInfluxDb};
use crate::settings;

#[derive(Clone, Debug)]
pub enum DataLoggerMessage {
    InfluxDbConnected,
    InfluxDbDisconnected,
    HassMqttConnected,
    HassMqttDisconnected,
}

pub struct DataLoggerActor {
    influxdb: Option<InfluxDBClient>,
    hassmqtt: Option<HassMqttClient>,
    influxdb_connected: bool,
    hassmqtt_connected: bool,
    rpict_rx: broadcast::Receiver<RpictMessage>,
    linky_rx: broadcast::Receiver<LinkyMessage>,
    tx: broadcast::Sender<DataLoggerMessage>,
}

#[derive(Clone)]
pub struct DataLoggerHandle {
    tx: broadcast::Sender<DataLoggerMessage>,
}

impl DataLoggerActor {
    /// Publishes a frame to every configured sink and emits a status message
    /// whenever a sink's connection state changes. Sinks are independent: a
    /// failure of one never affects the other.
    async fn publish_to_sinks<P>(&mut self, frame: &P)
    where
        P: ToInfluxDb + ToHassMqtt,
    {
        if let Some(client) = &self.influxdb {
            let connected = client.publish(frame).await.is_ok();
            if connected != self.influxdb_connected {
                self.influxdb_connected = connected;
                let msg = if connected { InfluxDbConnected } else { InfluxDbDisconnected };
                self.tx.send(msg).unwrap_or_default();
            }
        }
        if let Some(client) = &self.hassmqtt {
            // Non-blocking; connection truth comes from the MQTT event loop, not this call.
            let _ = client.publish(frame);
            let connected = client.is_connected();
            if connected != self.hassmqtt_connected {
                self.hassmqtt_connected = connected;
                let msg = if connected { HassMqttConnected } else { HassMqttDisconnected };
                self.tx.send(msg).unwrap_or_default();
            }
        }
    }

    async fn run(&mut self) {
        loop {
            tokio::select! {
                msg = self.rpict_rx.recv() => match msg {
                    Ok(RpictMessage::NewFrame(frame)) => {
                        log::trace!("New Rpict frame: {:?}", frame);
                        self.publish_to_sinks(&frame).await;
                    },
                    Err(RecvError::Lagged(skipped)) => {
                        log::warn!("Lag while logging rpict data, skipped {:?} frames", skipped);
                    },
                    _ => {}
                },
                msg = self.linky_rx.recv() => match msg {
                    Ok(LinkyMessage::NewFrame(frame)) => {
                        log::trace!("New Linky frame: {:?}", frame);
                        self.publish_to_sinks(&frame).await;
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
        hassmqtt_settings: &Option<settings::HassMqtt>,
        rpict: &RpictActorHandle,
        linky: &LinkyActorHandle,
    ) -> Result<DataLoggerHandle, Box<dyn Error>> {
        let influxdb = influxdb_settings
            .clone()
            .map(|settings| InfluxDBClient::new(&settings))
            .transpose()?;
        let hassmqtt = hassmqtt_settings
            .clone()
            .map(|settings| HassMqttClient::new(&settings))
            .transpose()?;
        let rpict_rx = rpict.subscribe();
        let linky_rx = linky.subscribe();
        // fork
        let (tx, _) = broadcast::channel(1);
        let mut actor = DataLoggerActor {
            influxdb,
            hassmqtt,
            influxdb_connected: false,
            hassmqtt_connected: false,
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
