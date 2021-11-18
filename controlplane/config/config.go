package config

import "errors"

type Config struct {
	PostgresHost string `mapstructure:"postgres_host"`
	PostgresPort string `mapstructure:"postgres_port"`
	PostgresSSL  bool   `mapstructure:"postgres_ssl"`
	DatabaseName string `mapstructure:"database_name"`
	ListenPort   string `mapstructure:"listen_port"`
	JwtKey       string `mapstructure:"jwt_key"`
}

func (c *Config) Validate() error {
	if c.PostgresHost == "" {
		return errors.New("postgres host is a required config")
	}
	if c.PostgresPort == "" {
		return errors.New("postgres port is a required config")
	}
	if c.DatabaseName == "" {
		return errors.New("database name is a required config")
	}
	if c.ListenPort == "" {
		c.ListenPort = ":3123"
	}
	if c.JwtKey == "" {
		return errors.New("jwt key is a required config")
	}
	return nil
}
