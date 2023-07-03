use std::result::Result;

use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Hmi {
    pub max_line_power_watts: f32,
    pub sleep_timeout_secs: u64,
    pub button_debounce_ms: u64,
    pub button_bcm_pin: u8,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Serial {
    pub rpict: String,
    pub linky: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct InfluxDB {
    pub host: String,
    pub port: usize,
    pub database: String,
    pub prefix: Option<String>,
}

impl InfluxDB {
    pub(crate) fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct HassMqtt {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub log_level: String,
    pub hmi: Hmi,
    pub serial: Serial,
    pub influxdb: Option<InfluxDB>,
    pub hassmqtt: Option<HassMqtt>,
}

impl Settings {
    pub fn new(yaml_config_opt: Option<String>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        const DEFAULTS: &str = include_str!("settings.default.yml");
        builder = builder.add_source(File::from_str(DEFAULTS, FileFormat::Yaml));
        if let Some(yaml_str) = yaml_config_opt {
            builder = builder.add_source(File::from_str(&yaml_str, FileFormat::Yaml));
        }
        // See https://github.com/mehcode/config-rs/issues/391
        let env = Environment::with_prefix("app").prefix_separator("__").separator("__");
        builder = builder.add_source(env);
        builder.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_default_settings() {
        let _ = Settings::new(None);
    }

    #[test]
    fn test_load_example_settings() {
        let example_settings = include_str!("settings.example.yml").to_string();
        let settings = Settings::new(Some(example_settings)).unwrap();
        assert!(settings.influxdb.is_some());
    }
}
