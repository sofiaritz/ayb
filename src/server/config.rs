use fernet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml;
use url::Url;

use crate::error::AybError;

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfigWeb {
    pub info_url: Url,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfigCors {
    pub origin: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfigAuthentication {
    pub fernet_key: String,
    pub token_expiration_seconds: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfigEmail {
    pub from: String,
    pub reply_to: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfigIsolation {
    pub nsjail_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AybConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub data_path: String,
    pub e2e_testing: Option<bool>,
    pub authentication: AybConfigAuthentication,
    pub email: AybConfigEmail,
    pub web: Option<AybConfigWeb>,
    pub cors: AybConfigCors,
    pub isolation: Option<AybConfigIsolation>,
}

impl AybConfig {
    pub fn e2e_testing_on(&self) -> bool {
        self.e2e_testing.unwrap_or(false)
    }
}

pub fn config_to_toml(ayb_config: AybConfig) -> Result<String, AybError> {
    Ok(toml::to_string(&ayb_config)?)
}

pub fn default_server_config() -> AybConfig {
    AybConfig {
        host: "0.0.0.0".to_string(),
        port: 5433,
        database_url: "sqlite://ayb_data/ayb.sqlite".to_string(),
        data_path: "./ayb_data".to_string(),
        e2e_testing: None,
        authentication: AybConfigAuthentication {
            fernet_key: fernet::Fernet::generate_key(),
            token_expiration_seconds: 3600,
        },
        email: AybConfigEmail {
            from: "Server Sender <server@example.org>".to_string(),
            reply_to: "Server Reply <replyto@example.org>".to_string(),
            smtp_host: "localhost".to_string(),
            smtp_port: 465,
            smtp_username: "login@example.org".to_string(),
            smtp_password: "the_password".to_string(),
        },
        cors: AybConfigCors {
            origin: "*".to_string(),
        },
        web: None,
        isolation: None,
    }
}

pub fn read_config(config_path: &PathBuf) -> Result<AybConfig, AybError> {
    let contents = fs::read_to_string(config_path)?;
    Ok(toml::from_str(&contents)?)
}
