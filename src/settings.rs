use std::ops::Deref;
use std::path::PathBuf;
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
pub struct Settings {
    pub log_level: String,
    pub hmi: Hmi,
    pub serial: Serial,
    pub influxdb: InfluxDB,
}

impl Settings {
    pub fn new(config_file_path: Option<&PathBuf>) -> Result<Self, ConfigError> {
        const DEFAULTS: &str = include_str!("settings.default.yml");
        let mut builder = Config::builder().add_source(File::from_str(DEFAULTS, FileFormat::Yaml));
        if let Some(path) = config_file_path {
            builder = builder.add_source(File::from(path.deref()).format(FileFormat::Yaml));
        }
        builder = builder.add_source(
            Environment::with_prefix("app")
                // See https://github.com/mehcode/config-rs/issues/391
                .prefix_separator("__")
                .separator("__"),
        );
        builder.build()?.try_deserialize()
    }
}
