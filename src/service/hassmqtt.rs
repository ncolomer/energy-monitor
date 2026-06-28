use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, LastWill, MqttOptions, Packet, QoS};
use serde_json::{json, Value};

use crate::driver::linky::LinkyFrame;
use crate::driver::rpict::RpictFrame;
use crate::settings;

const DEVICE_ID: &str = "energy-monitor";
const DEFAULT_DISCOVERY_PREFIX: &str = "homeassistant";
const SUPPORT_URL: &str = "https://github.com/ncolomer/energy-monitor";
const EXPIRE_AFTER_SECS: u32 = 60;
const CHANNEL_CAPACITY: usize = 64;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

#[derive(Debug, PartialEq)]
pub struct Sensor {
    pub name: String,
    pub source: &'static str,
    pub device_class: &'static str,
    pub state_class: Option<&'static str>,
    pub unit: Option<&'static str>,
    pub options: Option<&'static [&'static str]>,
}

impl Sensor {
    fn state_topic(&self) -> String {
        format!("{DEVICE_ID}/{}", self.source)
    }

    fn to_discovery_message(&self, discovery_prefix: &str, availability_topic: &str) -> Message {
        let topic = format!(
            "{discovery_prefix}/sensor/{DEVICE_ID}/{source}_{name}/config",
            source = self.source,
            name = self.name
        );
        let mut config = json!({
            "name": format!("{} {}", self.source, self.name),
            "unique_id": format!("{DEVICE_ID}_{}_{}", self.source, self.name),
            "state_topic": self.state_topic(),
            "value_template": format!("{{{{ value_json.{} }}}}", self.name),
            "availability_topic": availability_topic,
            "expire_after": EXPIRE_AFTER_SECS,
            "device_class": self.device_class,
            "device": {
                "identifiers": [DEVICE_ID],
                "name": DEVICE_ID,
                "manufacturer": "DIY",
                "model": DEVICE_ID,
                "sw_version": env!("CARGO_PKG_VERSION"),
            },
            "origin": {
                "name": DEVICE_ID,
                "sw_version": env!("CARGO_PKG_VERSION"),
                "support_url": SUPPORT_URL,
            },
        });
        if let Some(state_class) = self.state_class {
            config["state_class"] = Value::String(state_class.to_string());
        }
        if let Some(unit) = self.unit {
            config["unit_of_measurement"] = Value::String(unit.to_string());
        }
        if let Some(options) = self.options {
            config["options"] = Value::Array(options.iter().map(|o| Value::String(o.to_string())).collect());
        }
        let payload = serde_json::to_string(&config).unwrap();
        Message { topic, payload }
    }
}

pub trait ToHassMqtt {
    fn to_state_message(&self) -> Message;
}

impl RpictFrame {
    pub fn sensors() -> Vec<Sensor> {
        let mut sensors = Vec::with_capacity(15);
        for phase in ["l1", "l2", "l3"] {
            sensors.extend([
                Sensor { name: format!("{phase}_real_power"), source: "rpict", device_class: "power", state_class: Some("measurement"), unit: Some("W"), options: None },
                Sensor { name: format!("{phase}_apparent_power"), source: "rpict", device_class: "apparent_power", state_class: Some("measurement"), unit: Some("VA"), options: None },
                Sensor { name: format!("{phase}_irms"), source: "rpict", device_class: "current", state_class: Some("measurement"), unit: Some("A"), options: None },
                Sensor { name: format!("{phase}_vrms"), source: "rpict", device_class: "voltage", state_class: Some("measurement"), unit: Some("V"), options: None },
                Sensor { name: format!("{phase}_power_factor"), source: "rpict", device_class: "power_factor", state_class: Some("measurement"), unit: None, options: None },
            ]);
        }
        sensors
    }
}

impl ToHassMqtt for RpictFrame {
    fn to_state_message(&self) -> Message {
        Message {
            topic: format!("{DEVICE_ID}/rpict"),
            payload: serde_json::to_string(self).unwrap(),
        }
    }
}

impl LinkyFrame {
    pub fn sensors() -> Vec<Sensor> {
        vec![
            Sensor { name: "hchc".to_string(), source: "linky", device_class: "energy", state_class: Some("total_increasing"), unit: Some("Wh"), options: None },
            Sensor { name: "hchp".to_string(), source: "linky", device_class: "energy", state_class: Some("total_increasing"), unit: Some("Wh"), options: None },
            Sensor { name: "ptec".to_string(), source: "linky", device_class: "enum", state_class: None, unit: None, options: Some(&["HC", "HP"]) },
        ]
    }
}

impl ToHassMqtt for LinkyFrame {
    fn to_state_message(&self) -> Message {
        Message {
            topic: format!("{DEVICE_ID}/linky"),
            payload: serde_json::to_string(self).unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum HassMqttClientError {
    Publish,
}

pub struct HassMqttClient {
    client: AsyncClient,
    connected: Arc<AtomicBool>,
}

impl HassMqttClient {
    pub fn new(settings: &settings::HassMqtt) -> Result<HassMqttClient, Box<dyn Error>> {
        let discovery_prefix = settings
            .discovery_prefix
            .clone()
            .unwrap_or_else(|| DEFAULT_DISCOVERY_PREFIX.to_string());
        let availability_topic = format!("{DEVICE_ID}/availability");
        let status_topic = format!("{discovery_prefix}/status");

        let discovery: Vec<Message> = RpictFrame::sensors()
            .iter()
            .chain(LinkyFrame::sensors().iter())
            .map(|sensor| sensor.to_discovery_message(&discovery_prefix, &availability_topic))
            .collect();

        let mut mqtt_options = MqttOptions::new(DEVICE_ID, settings.host.clone(), settings.port);
        mqtt_options.set_keep_alive(Duration::from_secs(60));
        if let Some(username) = settings.username.clone() {
            mqtt_options.set_credentials(username, settings.password.clone().unwrap_or_default());
        }
        mqtt_options.set_last_will(LastWill::new(availability_topic.clone(), "offline", QoS::AtLeastOnce, true));

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, CHANNEL_CAPACITY);
        let connected = Arc::new(AtomicBool::new(false));

        let task_client = client.clone();
        let task_connected = connected.clone();
        tokio::task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        log::info!("Home Assistant MQTT connected");
                        task_connected.store(true, Ordering::Relaxed);
                        if let Err(e) = task_client.try_subscribe(status_topic.as_str(), QoS::AtLeastOnce) {
                            log::error!("Home Assistant status subscribe error: {e:?}");
                        }
                        announce(&task_client, &discovery, &availability_topic);
                    }
                    Ok(Event::Incoming(Packet::Publish(publish)))
                        if publish.topic == status_topic && publish.payload.as_ref() == b"online" =>
                    {
                        log::info!("Home Assistant birth message received, re-announcing entities");
                        announce(&task_client, &discovery, &availability_topic);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        if task_connected.swap(false, Ordering::Relaxed) {
                            log::warn!("Home Assistant MQTT disconnected");
                        }
                        log::debug!("Home Assistant MQTT eventloop error: {e:?}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(HassMqttClient { client, connected })
    }

    pub fn publish(&self, payload: &impl ToHassMqtt) -> Result<(), HassMqttClientError> {
        let Message { topic, payload } = payload.to_state_message();
        self.client.try_publish(topic, QoS::AtLeastOnce, false, payload).map_err(|e| {
            log::error!("Home Assistant MQTT publish error: {e:?}");
            HassMqttClientError::Publish
        })
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}

fn announce(client: &AsyncClient, discovery: &[Message], availability_topic: &str) {
    for Message { topic, payload } in discovery {
        if let Err(e) = client.try_publish(topic.as_str(), QoS::AtLeastOnce, true, payload.as_bytes()) {
            log::error!("Home Assistant discovery publish error: {e:?}");
        }
    }
    if let Err(e) = client.try_publish(availability_topic, QoS::AtLeastOnce, true, "online") {
        log::error!("Home Assistant availability publish error: {e:?}");
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2023-07-09T14:01:10Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn test_sensor_to_discovery_message() {
        // Given
        let sensor = Sensor {
            name: "l1_real_power".to_string(),
            source: "rpict",
            device_class: "power",
            state_class: Some("measurement"),
            unit: Some("W"),
            options: None,
        };
        // When
        let Message { topic, payload } = sensor.to_discovery_message("homeassistant", "energy-monitor/availability");
        // Then
        assert_eq!(topic, "homeassistant/sensor/energy-monitor/rpict_l1_real_power/config");
        let value: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(value["name"], "rpict l1_real_power");
        assert_eq!(value["unique_id"], "energy-monitor_rpict_l1_real_power");
        assert_eq!(value["state_topic"], "energy-monitor/rpict");
        assert_eq!(value["value_template"], "{{ value_json.l1_real_power }}");
        assert_eq!(value["availability_topic"], "energy-monitor/availability");
        assert_eq!(value["device_class"], "power");
        assert_eq!(value["state_class"], "measurement");
        assert_eq!(value["unit_of_measurement"], "W");
        assert_eq!(value["expire_after"], 60);
        assert_eq!(value["device"]["identifiers"][0], "energy-monitor");
        assert_eq!(value["device"]["sw_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["origin"]["support_url"], SUPPORT_URL);
    }

    #[test]
    fn test_sensor_without_unit_omits_unit_of_measurement() {
        // Given
        let sensor = Sensor {
            name: "l1_power_factor".to_string(),
            source: "rpict",
            device_class: "power_factor",
            state_class: Some("measurement"),
            unit: None,
            options: None,
        };
        // When
        let Message { payload, .. } = sensor.to_discovery_message("homeassistant", "energy-monitor/availability");
        // Then
        let value: Value = serde_json::from_str(&payload).unwrap();
        assert!(value.get("unit_of_measurement").is_none());
    }

    #[test]
    fn test_rpict_sensors() {
        // When
        let sensors = RpictFrame::sensors();
        // Then
        assert_eq!(sensors.len(), 15);
        assert!(sensors.iter().any(|s| s.name == "l1_real_power" && s.device_class == "power" && s.unit == Some("W")));
        assert!(sensors.iter().any(|s| s.name == "l3_power_factor" && s.device_class == "power_factor" && s.unit.is_none()));
    }

    #[test]
    fn test_linky_energy_sensors_are_total_increasing() {
        // When
        let energy: Vec<_> = LinkyFrame::sensors()
            .into_iter()
            .filter(|s| s.device_class == "energy")
            .collect();
        // Then
        assert_eq!(energy.len(), 2);
        assert!(energy
            .iter()
            .all(|s| { s.source == "linky" && s.state_class == Some("total_increasing") && s.unit == Some("Wh") }));
    }

    #[test]
    fn test_linky_ptec_is_enum_sensor() {
        // Given
        let ptec = LinkyFrame::sensors().into_iter().find(|s| s.name == "ptec").unwrap();
        // Then
        assert_eq!(ptec.device_class, "enum");
        assert_eq!(ptec.state_class, None);
        assert_eq!(ptec.unit, None);
        assert_eq!(ptec.options, Some(&["HC", "HP"][..]));
        let Message { payload, .. } = ptec.to_discovery_message("homeassistant", "energy-monitor/availability");
        let value: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(value["device_class"], "enum");
        assert_eq!(value["value_template"], "{{ value_json.ptec }}");
        assert!(value.get("state_class").is_none());
        assert!(value.get("unit_of_measurement").is_none());
        assert_eq!(value["options"], serde_json::json!(["HC", "HP"]));
    }

    #[test]
    fn test_rpictframe_to_state_message() {
        // Given
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
            timestamp: timestamp(),
        };
        // When
        let Message { topic, payload } = frame.to_state_message();
        // Then
        assert_eq!(topic, "energy-monitor/rpict");
        assert_eq!(
            payload,
            r#"{"node_id":11,"l1_real_power":-82.96,"l1_apparent_power":422.95,"l1_irms":1.64,"l1_vrms":257.65,"l1_power_factor":0.194,"l2_real_power":-50.23,"l2_apparent_power":144.52,"l2_irms":0.56,"l2_vrms":259.95,"l2_power_factor":0.346,"l3_real_power":24.55,"l3_apparent_power":47.17,"l3_irms":0.18,"l3_vrms":259.7,"l3_power_factor":0.509,"timestamp":"2023-07-09T14:01:10Z"}"#
        );
    }

    #[test]
    fn test_linkyframe_to_state_message() {
        // Given
        let frame = LinkyFrame {
            adco: "041876097767".to_string(),
            ptec: "HP".to_string(),
            hchc: 19_650_909,
            hchp: 43_280_553,
            timestamp: timestamp(),
        };
        // When
        let Message { topic, payload } = frame.to_state_message();
        // Then
        assert_eq!(topic, "energy-monitor/linky");
        assert_eq!(
            payload,
            r#"{"adco":"041876097767","ptec":"HP","hchc":19650909,"hchp":43280553,"timestamp":"2023-07-09T14:01:10Z"}"#
        );
    }
}
