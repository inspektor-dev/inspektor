// Copyright 2022 poonai
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::anyhow;
use serde::Deserialize;
use serde_yaml;

#[derive(Deserialize)]
pub struct Config {
    pub driver_type: Option<String>,
    pub controlplane_addr: Option<String>,
    pub postgres_config: Option<PostgresConfig>,
    pub secret_token: Option<String>,
}

impl Config {
    pub fn validate(&mut self) -> Result<(), anyhow::Error> {
        if let Some(driver_type) = &self.driver_type {
            if driver_type != "postgres" {
                return Err(anyhow!("only postgres driver supported"));
            }
        } else {
            return Err(anyhow!("driver_type is a required parameter"));
        }
        if self.controlplane_addr.is_none() {
            return Err(anyhow!("control plane address is a required parameter"));
        }
        if let Some(config) = &mut self.postgres_config {
            config.validate()?;
        } else {
            return Err(anyhow!("postgres_config is a required paremeter"));
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            driver_type: Some(String::from("postgres")),
            controlplane_addr: Some(String::from("localhost:5003")),
            postgres_config: Some(PostgresConfig::default()),
            secret_token: Some(String::from(
                "10c740e1d45eac77b6ff00c2211489bdf93f6a5eb7bc159a6b69b3e4f660",
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub target_addr: Option<String>,
    pub target_username: Option<String>,
    pub target_password: Option<String>,
    pub target_port: Option<String>,
    pub proxy_listen_port: Option<String>,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            target_addr: Some(String::from("localhost:5432")),
            target_username: Some(String::from("debuggeruser")),
            target_password: Some(String::from("debuggerpassword")),
            proxy_listen_port: Some(String::from("8080")),
            target_port: Some(String::from("5432"))
        }
    }
}

impl PostgresConfig {
    fn validate(&mut self) -> Result<(), anyhow::Error> {
        if self.target_addr.is_none() {
            return Err(anyhow!("target address is requrired parameter"));
        }
        if self.target_username.is_none() {
            return Err(anyhow!("target user name is requrired parameter"));
        }
        if self.target_password.is_none() {
            return Err(anyhow!("target password is requrired parameter"));
        }
        if self.target_port.is_none(){
            return Err(anyhow!("target port is a required parameter"));
        }
        if let None = self.proxy_listen_port {
            self.proxy_listen_port = Some("8080".to_string())
        }
        Ok(())
    }
}

pub fn read_config(config_path: &std::path::Path) -> Result<Config, anyhow::Error> {
    let config_buf = std::fs::read(config_path)
        .map_err(|e| anyhow!("error while reading config. err: {:?}", e))?;
    Ok(serde_yaml::from_slice::<Config>(&config_buf[..])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[test]
    fn test_config_basic() {
        let path = env::current_dir().unwrap();
        let mut config = read_config(&path.join("src/config/test_config.yaml")).unwrap();
        config.validate().unwrap();
    }
}
