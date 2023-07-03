use std::error::Error;

use std::time::Duration;

use async_stream::stream;
use chrono::{DateTime, Utc};

use regex::Regex;

use log::{error, LevelFilter};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};

use serde_json::Value;

use energy_monitor::driver::linky::LinkyFrame;
use energy_monitor::driver::rpict::RpictFrame;
use energy_monitor::service::hassmqtt::{HassMqttClient, Message, Sensor};
use energy_monitor::settings::HassMqtt;

use futures::stream::BoxStream;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

struct TestContext<'a> {
    test_id: String,
    messages: BoxStream<'a, Message>,
    settings: HassMqtt,
}

impl TestContext<'_> {
    async fn new<'a>() -> Result<TestContext<'a>, Box<dyn Error>> {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Info)
            .try_init();
        let test_id = Utc::now().timestamp_micros().to_string();
        // Create listener client
        const MQTT_HOST: &str = "test.mosquitto.org";
        const MQTT_PORT: u16 = 1883;
        let mut mqtt_options = MqttOptions::new(test_id.clone() + "/test-listener", MQTT_HOST, MQTT_PORT);
        mqtt_options.set_keep_alive(Duration::from_secs(60));
        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);
        client.subscribe(test_id.clone() + "/#", QoS::AtLeastOnce).await?;
        let _ = eventloop.poll().await; // warmup connection
        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(p))) => tx.send(p).unwrap(),
                    Err(e) => error!("MQTT error: {e:?}"),
                    _ => (),
                }
            }
        });
        let stream = stream! {
            while let Some(p) = rx.recv().await {
                yield Message {
                    topic: p.topic ,
                    payload: String::from_utf8(p.payload.to_vec()).unwrap()
                };
            }
        };

        Ok(TestContext {
            test_id,
            messages: Box::pin(stream),
            settings: HassMqtt {
                host: MQTT_HOST.to_string(),
                port: MQTT_PORT,
                username: None,
                password: None,
            },
        })
    }
}

lazy_static! {
    pub static ref TIMESTAMP: DateTime<Utc> = DateTime::parse_from_rfc3339("2023-07-09T14:01:10Z")
        .unwrap()
        .with_timezone(&Utc);
}

#[tokio::test]
async fn test_hassmqttclient_announce_sensors() {
    // Given
    let context = TestContext::new().await.unwrap();
    let client = HassMqttClient::new(&context.settings, Some(&context.test_id)).unwrap();
    let topic_regex = Regex::new(r"^\d+/homeassistant/sensor/energy-monitor/(?<sensor>[^/]+)/config$").unwrap();
    let mut sensors: Vec<Sensor> = RpictFrame::sensors()
        .into_iter()
        .chain(LinkyFrame::sensors().into_iter())
        .collect();
    let mut seen: Vec<Sensor> = vec![];
    // When
    client.announce().await;
    // Then
    let stream = context.messages.take_while(|m| m.topic.ends_with("/config"));
    tokio::pin!(stream);

    while let Some(Message { topic, payload }) = stream.next().await {
        assert!(topic_regex.is_match(&topic), "unexpected topic {}", topic);
        let sensor_name = topic_regex
            .captures(&topic)
            .and_then(|x| x.name("sensor"))
            .map(|x| x.as_str())
            .unwrap();
        let sensor = sensors
            .iter()
            .position(|s| s.name == sensor_name)
            .map(|i| seen.push(sensors.swap_remove(i)));
        assert!(sensor.is_some(), "unexpected sensor {}", sensor_name);
        let payload: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(
            format!("{}/energy-monitor/status", &context.test_id),
            payload["availability_topic"],
            "unexpected availability_topic"
        );
    }

    assert!(sensors.is_empty(), "sensor(s) was not announced: {:?}", sensors);
}

#[tokio::test]
async fn test_hassmqttclient_announce_online() {
    // Given
    let context = TestContext::new().await.unwrap();
    let client = HassMqttClient::new(&context.settings, Some(&context.test_id)).unwrap();
    // When
    client.announce().await;
    // Then
    let stream = context.messages.skip_while(|m| m.topic.ends_with("/config"));
    tokio::pin!(stream);
    let topic = format!("{}/energy-monitor/status", &context.test_id);
    let payload = "online".to_string();
    assert_eq!(
        Some(Message { topic, payload }),
        stream.next().await,
        "unexpected message"
    );
}

#[tokio::test]
async fn test_hassmqttclient_publish_rpictframe() {
    // Given
    let context = TestContext::new().await.unwrap();
    let client = HassMqttClient::new(&context.settings, Some(&context.test_id)).unwrap();
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
    client.publish(&frame).await.unwrap();
    // Then
    let stream = context.messages;
    tokio::pin!(stream);
    let topic = format!("{}/energy-monitor/rpict", &context.test_id);
    let payload = r#"{"node_id":11,"l1_real_power":-82.96,"l1_apparent_power":422.95,"l1_irms":1.64,"l1_vrms":257.65,"l1_power_factor":0.194,"l2_real_power":-50.23,"l2_apparent_power":144.52,"l2_irms":0.56,"l2_vrms":259.95,"l2_power_factor":0.346,"l3_real_power":24.55,"l3_apparent_power":47.17,"l3_irms":0.18,"l3_vrms":259.7,"l3_power_factor":0.509,"timestamp":"2023-07-09T14:01:10Z"}"#.to_string();
    assert_eq!(
        Some(Message { topic, payload }),
        stream.next().await,
        "unexpected message"
    );
}

#[tokio::test]
async fn test_hassmqttclient_publish_linkyframe() {
    // Given
    let context = TestContext::new().await.unwrap();
    let client = HassMqttClient::new(&context.settings, Some(&context.test_id)).unwrap();
    let frame = LinkyFrame {
        adco: "041876097767".to_string(),
        ptec: "HP".to_string(),
        hchc: 19_650_909,
        hchp: 43_280_553,
        timestamp: *TIMESTAMP,
    };
    // When
    client.publish(&frame).await.unwrap();
    // Then
    let stream = context.messages;
    tokio::pin!(stream);
    let topic = format!("{}/energy-monitor/linky", &context.test_id);
    let payload =
        r#"{"adco":"041876097767","ptec":"HP","hchc":19650909,"hchp":43280553,"timestamp":"2023-07-09T14:01:10Z"}"#
            .to_string();
    assert_eq!(
        Some(Message { topic, payload }),
        stream.next().await,
        "unexpected message"
    );
}
