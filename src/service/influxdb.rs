use std::error::Error;

use reqwest;

use crate::driver::linky::LinkyFrame;
use crate::driver::rpict::RpictFrame;
use crate::settings;

pub struct InfluxDBClient {
    client: reqwest::Client,
    settings: settings::InfluxDB,
}

#[derive(Debug)]
pub enum InfluxDBClientError {
    UnexpectedResponse,
    TimedOut,
    Unknown,
}

impl InfluxDBClient {
    pub fn new(settings: &settings::InfluxDB) -> Result<InfluxDBClient, Box<dyn Error>> {
        let settings = settings.clone();
        let client = reqwest::Client::new();
        Ok(InfluxDBClient { client, settings })
    }

    pub async fn publish(&self, payload: &impl InfluxDbSerialize) -> Result<(), InfluxDBClientError> {
        // https://docs.influxdata.com/influxdb/v1.8/tools/api/#write-http-endpoint
        let write_url = format!("{}/write", self.settings.base_url());
        let request = self
            .client
            .post(write_url)
            .query(&[("db", &self.settings.database.as_str()), ("precision", &"ms")])
            .body(payload.to_line_data(&self.settings.prefix))
            .send();
        match request.await {
            Ok(res) if res.status() == 204 => Ok(()),
            Ok(res) => {
                log::error!("Unexpected response: {res:?}");
                Err(InfluxDBClientError::UnexpectedResponse)
            }
            Err(err) if err.is_timeout() => Err(InfluxDBClientError::TimedOut),
            Err(_) => Err(InfluxDBClientError::Unknown),
        }
    }
}

pub trait InfluxDbSerialize {
    // https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/
    fn to_line_data(&self, prefix: &Option<String>) -> String;
}

impl InfluxDbSerialize for RpictFrame {
    fn to_line_data(&self, prefix: &Option<String>) -> String {
        let measurement = vec![prefix.clone(), Some("rpict".to_string())]
            .iter()
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
        ]
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<String>>()
        .join(",");
        let timestamp = self.timestamp.timestamp_millis();
        format!("{measurement},{tags} {fields} {timestamp}")
    }
}

impl InfluxDbSerialize for LinkyFrame {
    fn to_line_data(&self, prefix: &Option<String>) -> String {
        let measurement = vec![prefix.clone(), Some("linky".to_string())]
            .iter()
            .filter_map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(".");
        let tags = format!("adco={}", self.adco);
        let fields = vec![("hc_index", self.hchc), ("hp_index", self.hchp)]
            .iter()
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
            timestamp: now,
        };
        // When
        let actual = frame.to_line_data(&Some("prefix".to_string()));
        // Then
        assert_eq!(
            actual,
            "prefix.rpict,node_id=11 \
        l1_real_power=-82.96,l1_apparent_power=422.95,l1_irms=1.64,l1_vrms=257.65,l1_power_factor=0.194,\
        l2_real_power=-50.23,l2_apparent_power=144.52,l2_irms=0.56,l2_vrms=259.95,l2_power_factor=0.346,\
        l3_real_power=24.55,l3_apparent_power=47.17,l3_irms=0.18,l3_vrms=259.7,l3_power_factor=0.509 1657113606"
        );
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
            timestamp: now,
        };
        // When
        let actual = frame.to_line_data(&Some("prefix".to_string()));
        // Then
        assert_eq!(
            actual,
            "prefix.linky,adco=041876097767 hc_index=19650909,hp_index=43280553 1657113606"
        );
    }
}
