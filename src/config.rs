use homedir::my_home;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

pub fn get_config() -> AppConfig {
    let config = match load_or_initialize() {
        Ok(v) => v,
        Err(err) => {
            match err {
                ConfigError::IoError(err) => {
                    eprintln!("An error occurred while loading the config: {err}");
                }
                ConfigError::InvalidConfig(err) => {
                    eprintln!("An error occurred while parsing the config:");
                    eprintln!("{err}");
                }
            }

            AppConfig {
                db_name: "dbsql1".to_string(),
                db_user: "dbsql1".to_string(),
                db_pass: "passpass".to_string(),
                db_host: "localhost".to_string(),
                db_port: "3306".to_string(),
                wait_min: 4000,
                wait_max: 10000,
            }
        }
    };
    //println!("{:?}", config);
    return config;
    //    return "xxxx".to_string();
}

enum ConfigError {
    IoError(io::Error),
    InvalidConfig(toml::de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::InvalidConfig(value)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_host: String,
    pub db_port: String,
    pub wait_min: u64,
    pub wait_max: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_name: "dbsql1".to_string(),
            db_user: "dbsql1".to_string(),
            db_pass: "passpass".to_string(),
            db_host: "localhost".to_string(),
            db_port: "3306".to_string(),
            wait_min: 4000,
            wait_max: 10000,
        }
    }
}

fn load_or_initialize() -> Result<AppConfig, ConfigError> {
    //  https://dev.to/zofia/why-do-we-need-configuration-creating-and-handling-configuration-files-in-rust-4a46?ysclid=m00bsa1iuz12379992
    let home = my_home().unwrap().unwrap();
    let _config_dir = &format!("{0}/.mtranslate/", home.display());
    let _config_path = &format!("{0}/.mtranslate/config.toml", home.display());
    let config_path = Path::new(_config_path);
    let config_dir = Path::new(_config_dir);
    let config = AppConfig::default();

    if config_path.exists() {
        //println!("...path exists:{}", _config_path);
        let content = fs::read_to_string(config_path)?;
        //println!(":{:?}", content);
        let config: AppConfig = toml::from_str(&content).expect("failed");
        return Ok(config);
    } else {
        let toml = toml::to_string(&config).unwrap();
        //println!(":{:?}", toml);
        let _x = fs::create_dir_all(config_dir);
        fs::write(config_path, toml)?;
    }

    //    println!(":{:?}", config.host);
    Ok(config)
}
