pub struct Config {
    pub driver_type: Option<String>,
    pub controlplane_addr: Option<String>,
    pub postgres_config: Option<PostgresConfig>,
    pub secret_token: Option<String>,
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

#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub target_addr: Option<String>,
    pub target_username: Option<String>,
    pub target_password: Option<String>,
    pub proxy_listen_port: Option<String>,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            target_addr: Some(String::from("localhost:5432")),
            target_username: Some(String::from("debuggeruser")),
            target_password: Some(String::from("debuggerpassword")),
            proxy_listen_port: Some(String::from("8080")),
        }
    }
}
