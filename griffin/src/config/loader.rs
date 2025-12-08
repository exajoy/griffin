use super::config::Config;
use anyhow::Result;
use std::path::Path;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn from_file(path: &Path) -> Result<Config> {
        let txt = std::fs::read_to_string(path)?;
        let cfg = serde_yaml::from_str(&txt)?;
        Ok(cfg)
    }
}
