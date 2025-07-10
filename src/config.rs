use crate::{error, info, wait_exit};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub dir: String,
    pub name_exe: String,
    pub name_process: String,
}

impl Default for Config {
    fn default() -> Self {
        let current_dir = std::env::current_dir()
            .expect("Failed to get current directory")
            .to_string_lossy()
            .to_string();
        let path = Path::new(&current_dir);

        Self {
            dir: path.to_string_lossy().to_string(),
            name_exe: "example.exe".to_string(),
            name_process: "example.exe".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let current_path = std::env::current_dir()
            .expect("Failed to get current directory")
            .to_string_lossy()
            .to_string();
        let config_path = Path::new(&current_path).join("launcher.toml");
        let config = match std::fs::read_to_string(&config_path) {
            Ok(content) => toml::from_str::<Config>(&content),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    info!(
                        "Configuration file not found at: {}, creating default",
                        config_path.display()
                    );
                    let default = Config::default();
                    match std::fs::write(&config_path, toml::to_string_pretty(&default)?) {
                        Ok(_) => {
                            info!(
                                "Default configuration created at: {}",
                                config_path.display()
                            );
                            wait_exit(0);
                        }
                        Err(e) => {
                            error!("Failed to write default configuration: {}", e);
                            wait_exit(1);
                        }
                    }
                }
                _ => {
                    error!("Failed to read configuration file: {}", err);
                    wait_exit(1);
                }
            },
        }?;

        Ok(config)
    }

    pub fn exe_path(&self) -> PathBuf {
        Path::new(&self.dir).join(&self.name_exe)
    }
}
