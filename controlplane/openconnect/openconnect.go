// Copyright 2022 Balaji (rbalajis25@gmail.com)
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

package openconnect

import (
	"inspektor/config"
	"inspektor/utils"

	"go.uber.org/zap"
	"golang.org/x/oauth2"
)

type OpenConnect interface {
	GetConfig() *oauth2.Config
	GetUserName(token *oauth2.Token) string
}

var OpenConnectRegistry = map[string]func(cfg *config.Config) OpenConnect{"github": NewGithubProvider}

func getBaseConfig(cfg *config.Config) *oauth2.Config {
	return &oauth2.Config{
		ClientID:     cfg.IdpClientID,
		ClientSecret: cfg.IdpClientSecret,
	}
}

func GetOpenConnectClient(cfg *config.Config) OpenConnect {
	configProvider, ok := OpenConnectRegistry[cfg.IdpProvider]
	if !ok {
		utils.Logger.Fatal("error while retriving config Provide", zap.String("provider_name", cfg.IdpProvider))
	}
	return configProvider(cfg)
}
