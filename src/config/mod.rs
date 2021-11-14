pub struct Config {
    driver_type: Option<String>,
    dataplane_addr: Option<String>,
    postgres_config: Option<PostgresConfig>,
    secret_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            driver_type: Some(String::from("postgres")),
            dataplane_addr: None,
            postgres_config: Some(PostgresConfig::default()),
            secret_token: None,
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
