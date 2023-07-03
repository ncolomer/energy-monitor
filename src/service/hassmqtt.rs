use rumqttc::{AsyncClient, LastWill, MqttOptions, QoS};

use serde_json::json;

use std::error::Error;
use std::time::Duration;

use crate::driver::linky::LinkyFrame;
use crate::driver::rpict::RpictFrame;
use crate::settings;

const DEVICE_NAME: &str = "energy-monitor";

pub struct HassMqttClient {
    client: AsyncClient,
    topic_prefix: String,
    discovery_topic_prefix: String,
    status_topic: String,
}

#[derive(Debug)]
pub enum HassMqttClientError {
    Publish,
}

impl HassMqttClient {
    pub fn new(settings: &settings::HassMqtt, topic_prefix: Option<&String>) -> Result<HassMqttClient, Box<dyn Error>> {
        let topic_prefix = topic_prefix
            .filter(|s| !s.is_empty())
            .map_or(String::from(""), |prefix| prefix.clone() + "/");
        let discovery_topic_prefix = topic_prefix.clone() + "homeassistant/";
        let status_topic = topic_prefix.clone() + DEVICE_NAME + "/status";

        let settings = settings.clone();
        let client_id = topic_prefix.clone() + DEVICE_NAME;
        let mut mqtt_options = MqttOptions::new(client_id, settings.host, settings.port);
        if let (Some(username), Some(password)) = (settings.username, settings.password) {
            mqtt_options.set_credentials(username, password);
        }
        mqtt_options.set_keep_alive(Duration::from_secs(60));
        mqtt_options.set_last_will(LastWill::new(&status_topic, "offline", QoS::AtLeastOnce, false));
        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 1);

        tokio::task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("MQTT error = {e:?}");
                        return;
                    }
                }
            }
        });

        Ok(HassMqttClient {
            client,
            topic_prefix,
            discovery_topic_prefix,
            status_topic,
        })
    }

    pub async fn announce(&self) {
        let mut sensors: Vec<Sensor> = vec![];
        sensors.extend(RpictFrame::sensors());
        sensors.extend(LinkyFrame::sensors());
        for sensor in sensors {
            let Message { topic, payload } = sensor.to_config_message(&self.status_topic);
            let topic = self.discovery_topic_prefix.clone() + topic.as_str();
            self.client
                .publish(&topic, QoS::AtLeastOnce, false, payload)
                .await
                .unwrap();
        }
        self.client
            .publish(&self.status_topic, QoS::AtLeastOnce, false, "online")
            .await
            .unwrap();
    }

    pub async fn publish(&self, payload: &impl Publishable) -> Result<(), HassMqttClientError> {
        let Message { topic, payload } = payload.to_state_message();
        let topic = self.topic_prefix.clone() + topic.as_str();
        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
            .map_err(|_| HassMqttClientError::Publish)
    }
}

#[derive(Debug, PartialEq)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

#[derive(Debug)]
pub struct Sensor {
    pub name: String,
    pub source: String,
    pub device_class: String,
    pub state_class: String,
    pub unit_of_measurement: String,
}

impl Sensor {
    fn to_config_message(&self, status_topic: &str) -> Message {
        let topic = format!("sensor/energy-monitor/{}/config", self.name);
        let payload = serde_json::to_string(&json!({
          "availability_topic": status_topic,
          "state_topic": format!("energy-monitor/{source}", source = self.source),
          "value_template": format!("{{{{ value_json.{sensor} }}}}", sensor = self.name),
          "device": {
            "name": DEVICE_NAME,
            "manufacturer": "DIY",
            "sw_version": env!("CARGO_PKG_VERSION")
          },
          "unique_id": format!("energy-monitor_{}_{}", self.source, self.name),
          "name": format!("energy-monitor {} {}", self.source, self.name),
          "device_class": self.device_class,
          "state_class": self.state_class,
          "unit_of_measurement": self.unit_of_measurement,
          "enabled_by_default": true,
          "entity_category": "diagnostic",
          "icon": "mdi:lightning-bolt"
        }))
        .unwrap();
        Message { topic, payload }
    }
}

pub trait Publishable {
    fn to_state_message(&self) -> Message;
}

impl RpictFrame {
    pub fn sensors() -> Vec<Sensor> {
        // See documentation: http://lechacal.com/wiki/index.php/RPICT3V1
        vec![
            Sensor {
                name: "l1_real_power".to_string(),
                source: "rpict".to_string(),
                device_class: "power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "W".to_string(),
            },
            Sensor {
                name: "l1_apparent_power".to_string(),
                source: "rpict".to_string(),
                device_class: "apparent_power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "VA".to_string(),
            },
            Sensor {
                name: "l1_irms".to_string(),
                source: "rpict".to_string(),
                device_class: "current".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "A".to_string(),
            },
            Sensor {
                name: "l1_vrms".to_string(),
                source: "rpict".to_string(),
                device_class: "voltage".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "V".to_string(),
            },
            Sensor {
                name: "l1_power_factor".to_string(),
                source: "rpict".to_string(),
                device_class: "power_factor".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "%".to_string(),
            },
            Sensor {
                name: "l2_real_power".to_string(),
                source: "rpict".to_string(),
                device_class: "power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "W".to_string(),
            },
            Sensor {
                name: "l2_apparent_power".to_string(),
                source: "rpict".to_string(),
                device_class: "apparent_power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "VA".to_string(),
            },
            Sensor {
                name: "l2_irms".to_string(),
                source: "rpict".to_string(),
                device_class: "current".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "A".to_string(),
            },
            Sensor {
                name: "l2_vrms".to_string(),
                source: "rpict".to_string(),
                device_class: "voltage".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "V".to_string(),
            },
            Sensor {
                name: "l2_power_factor".to_string(),
                source: "rpict".to_string(),
                device_class: "power_factor".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "%".to_string(),
            },
            Sensor {
                name: "l3_real_power".to_string(),
                source: "rpict".to_string(),
                device_class: "power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "W".to_string(),
            },
            Sensor {
                name: "l3_apparent_power".to_string(),
                source: "rpict".to_string(),
                device_class: "apparent_power".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "VA".to_string(),
            },
            Sensor {
                name: "l3_irms".to_string(),
                source: "rpict".to_string(),
                device_class: "current".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "A".to_string(),
            },
            Sensor {
                name: "l3_vrms".to_string(),
                source: "rpict".to_string(),
                device_class: "voltage".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "V".to_string(),
            },
            Sensor {
                name: "l3_power_factor".to_string(),
                source: "rpict".to_string(),
                device_class: "power_factor".to_string(),
                state_class: "measurement".to_string(),
                unit_of_measurement: "%".to_string(),
            },
        ]
    }
}

impl Publishable for RpictFrame {
    fn to_state_message(&self) -> Message {
        let topic = String::from("energy-monitor/rpict");
        let payload = serde_json::to_string(self).unwrap();
        Message { topic, payload }
    }
}

impl LinkyFrame {
    pub fn sensors() -> Vec<Sensor> {
        vec![
            Sensor {
                name: "hchp".to_string(),
                source: "linky".to_string(),
                device_class: "energy".to_string(),
                state_class: "total".to_string(),
                unit_of_measurement: "Wh".to_string(),
            },
            Sensor {
                name: "hchc".to_string(),
                source: "linky".to_string(),
                device_class: "energy".to_string(),
                state_class: "total".to_string(),
                unit_of_measurement: "Wh".to_string(),
            },
        ]
    }
}

impl Publishable for LinkyFrame {
    fn to_state_message(&self) -> Message {
        let topic = String::from("energy-monitor/linky");
        let payload = serde_json::to_string(self).unwrap();
        Message { topic, payload }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    lazy_static! {
        pub static ref TIMESTAMP: DateTime<Utc> = DateTime::parse_from_rfc3339("2023-07-09T14:01:10Z")
            .unwrap()
            .with_timezone(&Utc);
    }

    #[test]
    fn test_sensorconfig_to_discovery_message() {
        // Given
        let now = Sensor {
            name: "sensor".to_string(),
            source: "source".to_string(),
            device_class: "device_class".to_string(),
            state_class: "state_class".to_string(),
            unit_of_measurement: "unit_of_measurement".to_string(),
        };
        // When
        let Message { topic, payload } = now.to_config_message("status");
        // Then
        assert_eq!(topic, "sensor/energy-monitor/sensor/config");
        assert_eq!(
            payload,
            r#"{"availability_topic":"status","device":{"manufacturer":"DIY","name":"energy-monitor","sw_version":"0.1.0"},"device_class":"device_class","enabled_by_default":true,"entity_category":"diagnostic","icon":"mdi:lightning-bolt","name":"energy-monitor source sensor","state_class":"state_class","state_topic":"energy-monitor/source","unique_id":"energy-monitor_source_sensor","unit_of_measurement":"unit_of_measurement","value_template":"{{ value_json.sensor }}"}"#
        );
    }

    #[test]
    fn test_linkyframe_to_message() {
        // Given
        let frame = LinkyFrame {
            adco: "041876097767".to_string(),
            ptec: "HP".to_string(),
            hchc: 19_650_909,
            hchp: 43_280_553,
            timestamp: *TIMESTAMP,
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

    #[test]
    fn test_rpictframe_to_message() {
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
            l3_vrms: 259.7,
            l3_power_factor: 0.509,
            timestamp: *TIMESTAMP,
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
}
