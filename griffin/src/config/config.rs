use anyhow::Result;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    #[cfg(test)]
    pub message: String,

    pub listen_host: String,
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
}

impl Config {
    #[cfg(test)]
    pub fn with_message(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
    pub fn from_file(path: &Path) -> Result<Self> {
        let txt = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&txt)?;
        Ok(config)
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(test)]
            message: String::new(),

            listen_host: "127.0.0.1".into(),
            listen_port: 8080,
            target_host: "127.0.0.1".into(),
            target_port: 3000,
        }
    }
}
