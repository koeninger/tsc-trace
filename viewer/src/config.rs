use config::builder::DefaultState;
use config::{Config, ConfigBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn config_builder() -> ConfigBuilder<DefaultState> {
    Config::builder().add_source(config::File::new("config.js", config::FileFormat::Json5))
}

pub fn config() -> ViewerConfig {
    let cfg = config_builder()
        .build()
        .and_then(|b| b.try_deserialize::<ViewerConfig>())
        .expect("tag name configuration failed");
    cfg
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ViewerConfig {
    pub tag_names: Option<HashMap<u64, String>>,
    pub default_args: [String; 4],
}
