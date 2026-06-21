use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    3000
}

pub fn load() -> Result<AppConfig, ConfigError> {
    Config::builder()
        .add_source(Environment::with_prefix("APP"))
        .build()
        .map(|s| {
            s.try_deserialize()
                .expect("couldn't convert into AppConfig")
        })
}
