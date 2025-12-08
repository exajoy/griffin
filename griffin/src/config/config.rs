use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    #[cfg(test)]
    pub message: String,

    pub proxy_host: String,
    pub proxy_port: u16,
    pub forward_host: String,
    pub forward_port: u16,
}
impl Config {
    #[cfg(test)]
    pub fn with_message(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(test)]
            message: String::new(),

            proxy_host: "127.0.0.1".into(),
            proxy_port: 8080,
            forward_host: "127.0.0.1".into(),
            forward_port: 3000,
        }
    }
}
