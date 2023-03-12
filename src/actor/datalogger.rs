use std::cmp::max;
use std::error::Error;
use std::time::Duration;

use reqwest;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;

use DataLoggerMessage::*;

use crate::actor::linky::{LinkyActorHandle, LinkyMessage};
use crate::actor::rpict::{RpictActorHandle, RpictMessage};
use crate::driver::linky::LinkyFrame;
use crate::driver::rpict::RpictFrame;
use crate::settings;

#[derive(Clone, Debug)]
pub enum DataLoggerMessage {
    Connected,
    Disconnected,
}

pub struct DataLoggerActor {
    settings: settings::InfluxDB,
    rpict_rx: broadcast::Receiver<RpictMessage>,
    linky_rx: broadcast::Receiver<LinkyMessage>,
    client: reqwest::Client,
    tx: broadcast::Sender<DataLoggerMessage>,
}

#[derive(Clone)]
pub struct DataLoggerHandle {
    tx: broadcast::Sender<DataLoggerMessage>
}

impl DataLoggerActor {

    async fn wait_for_connectivity(&self) {
        // https://docs.influxdata.com/influxdb/v1.8/tools/api/#ping-http-endpoint
        const MAX_WAIT: Duration = Duration::from_secs(600);
        let ping_url = format!("{}/ping", self.settings.base_url());
        let mut retry_delay = Duration::from_secs(1);
        let mut latch = true;
        loop {
            match self.client.get(&ping_url).send().await {
                Ok(res) if res.status() == 204 => {
                    self.tx.send(Connected).unwrap_or_default();
                    return
                }
                _ => {
                    if latch { self.tx.send(Disconnected).unwrap_or_default(); latch = false; }
                    sleep(retry_delay).await;
                    retry_delay = max(retry_delay * 2, MAX_WAIT);
                }
            }
        }
    }

    async fn send_measurements(&self, payload: impl InfluxDbSerialize) {
        // https://docs.influxdata.com/influxdb/v1.8/tools/api/#write-http-endpoint
        let write_url = format!("{}/write", self.settings.base_url());
        let request = self.client.post(write_url)
            .query(&[("db", &self.settings.database.as_str()), ("precision", &"ms")])
            .body(payload.to_line_data(&self.settings.prefix))
            .send();
        match request.await {
            Ok(res) if res.status() == 204 => return,
            Ok(res) => log::error!("Unexpected HTTP response: {res:?}"),
            Err(err) if err.is_timeout() => {
                self.tx.send(Disconnected).unwrap_or_default();
                self.wait_for_connectivity().await;
            },
            Err(_) => {}
        }
    }

    async fn run(&mut self) {
        self.wait_for_connectivity().await;

        loop {
            tokio::select! {
                msg = self.rpict_rx.recv() => match msg {
                    Ok(RpictMessage::NewFrame(frame)) => {
                        log::trace!("New Rpict frame: {:?}", frame);
                        self.send_measurements(frame).await;
                    },
                    Err(RecvError::Lagged(skipped)) => {
                        log::warn!("Lag while logging rpict data, skipped {:?} frames", skipped);
                    },
                    _ => {}
                },
                msg = self.linky_rx.recv() => match msg {
                    Ok(LinkyMessage::NewFrame(frame)) => {
                        log::trace!("New Linky frame: {:?}", frame);
                        self.send_measurements(frame).await;
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

    pub fn new(
        settings: &settings::InfluxDB,
        rpict: &RpictActorHandle,
        linky: &LinkyActorHandle,
    ) -> Result<DataLoggerHandle, Box<dyn Error>>
    {
        let settings = settings.clone();
        let rpict_rx = rpict.subscribe();
        let linky_rx = linky.subscribe();
        let client = reqwest::Client::new();
        // fork
        let (tx, _) = broadcast::channel(1);
        let mut actor = DataLoggerActor { settings, rpict_rx, linky_rx, client, tx: tx.clone() };
        tokio::task::spawn(async move { actor.run().await });
        Ok(DataLoggerHandle { tx: tx.clone() })
    }

}

impl DataLoggerHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<DataLoggerMessage> {
        self.tx.subscribe()
    }
}

trait InfluxDbSerialize {
    // https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/
    fn to_line_data(&self, prefix: &Option<String>) -> String;
}

impl InfluxDbSerialize for RpictFrame {
    fn to_line_data(&self, prefix: &Option<String>) -> String {
        let measurement = vec![prefix.clone(), Some("rpict".to_string())].iter()
            .filter_map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(".");
        let tags = format!("node_id={}", self.node_id);
        let fields = vec![
            ("l1_real_power", self.l1_real_power),
            ("l1_apparent_power", self.l1_apparent_power),
            ("l1_irms", self.l1_irms),
            ("l1_vrms", self.l1_vrms),
            ("l1_power_factor", self.l1_power_factor),
            ("l2_real_power", self.l2_real_power),
            ("l2_apparent_power", self.l2_apparent_power),
            ("l2_irms", self.l2_irms),
            ("l2_vrms", self.l2_vrms),
            ("l2_power_factor", self.l2_power_factor),
            ("l3_real_power", self.l3_real_power),
            ("l3_apparent_power", self.l3_apparent_power),
            ("l3_irms", self.l3_irms),
            ("l3_vrms", self.l3_vrms),
            ("l3_power_factor", self.l3_power_factor),
        ].iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<String>>()
            .join(",");
        let timestamp = self.timestamp.timestamp_millis();
        format!("{measurement},{tags} {fields} {timestamp}")
    }
}

impl InfluxDbSerialize for LinkyFrame {
    fn to_line_data(&self, prefix: &Option<String>) -> String {
        let measurement = vec![prefix.clone(), Some("linky".to_string())].iter()
            .filter_map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(".");
        let tags = format!("adco={}", self.adco);
        let fields = vec![
            ("hc_index", self.hchc),
            ("hp_index", self.hchp),
        ].iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<String>>()
            .join(",");
        let timestamp = self.timestamp.timestamp_millis();
        format!("{measurement},{tags} {fields} {timestamp}")
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_influxdb_serialization_rpictframe() {
        // Given
        let now = Utc.timestamp_millis_opt(1657113606).unwrap();
        let frame = RpictFrame {
            node_id: 11,
            l1_real_power: -82.96,
            l1_apparent_power: 422.95,
            l1_irms: 1.64,
            l1_vrms: 257.65,
            l1_power_factor: 0.194,
            l2_real_power: -50.23,
            l2_apparent_power: 144.52,
            l2_irms: 0.56,
            l2_vrms: 259.95,
            l2_power_factor: 0.346,
            l3_real_power: 24.55,
            l3_apparent_power: 47.17,
            l3_irms: 0.18,
            l3_vrms: 259.70,
            l3_power_factor: 0.509,
            timestamp: now
        };
        // When
        let actual = frame.to_line_data(&Some("prefix".to_string()));
        // Then
        assert_eq!(actual, "prefix.rpict,node_id=11 \
        l1_real_power=-82.96,l1_apparent_power=422.95,l1_irms=1.64,l1_vrms=257.65,l1_power_factor=0.194,\
        l2_real_power=-50.23,l2_apparent_power=144.52,l2_irms=0.56,l2_vrms=259.95,l2_power_factor=0.346,\
        l3_real_power=24.55,l3_apparent_power=47.17,l3_irms=0.18,l3_vrms=259.7,l3_power_factor=0.509 1657113606");
    }

    #[test]
    fn test_influxdb_serialization_linkyframe() {
        // Given
        let now = Utc.timestamp_millis_opt(1657113606).unwrap();
        let frame = LinkyFrame {
            adco: "041876097767".to_string(),
            ptec: "HP".to_string(),
            hchc: 19_650_909,
            hchp: 43_280_553,
            timestamp: now
        };
        // When
        let actual = frame.to_line_data(&Some("prefix".to_string()));
        // Then
        assert_eq!(actual, "prefix.linky,adco=041876097767 hc_index=19650909,hp_index=43280553 1657113606");
    }

}
