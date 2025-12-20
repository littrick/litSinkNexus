use crate::internal::WarnExt;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::RwLock};
use tracing::log;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    auto_connect: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { auto_connect: true }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub file_path: PathBuf,
    config: RwLock<Config>,
}

impl AppConfig {
    pub fn parse(path: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&path)?;
        let config = toml::from_str(&content)?;

        Ok(Self {
            file_path: path,
            config: RwLock::new(config),
        })
    }
    pub fn parse_or_default(path: PathBuf) -> Self {
        match Self::parse(path.clone()) {
            Ok(config) => config,
            Err(e) => {
                log::warn!(
                    "Failed to parse config file {:?}, using default config: {:?}",
                    path,
                    e
                );
                Self {
                    file_path: path,
                    config: RwLock::new(Config::default()),
                }
            }
        }
    }
    pub fn auto_connect(&self) -> bool {
        self.config.read().unwrap().auto_connect
    }
    pub fn set_auto_connect(&self, value: bool) {
        let mut config = self.config.write().unwrap();
        config.auto_connect = value;
        let config_content = toml::to_string(&*config).unwrap();
        fs::write(&self.file_path, config_content).warn("Failed to write config file");
    }
}
