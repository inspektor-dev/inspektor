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
	"context"
	"inspektor/config"
	"inspektor/utils"

	githubClient "github.com/google/go-github/v42/github"
	"go.uber.org/zap"
	"golang.org/x/oauth2"
	"golang.org/x/oauth2/github"
)

type GithubOpenConnect struct {
	cfg *oauth2.Config
}

func NewGithubProvider(cfg *config.Config) OpenConnect {
	oCfg := getBaseConfig(cfg)
	oCfg.Scopes = []string{"admin:org:read", "user:email"}
	oCfg.Endpoint = github.Endpoint
	return &GithubOpenConnect{
		cfg: oCfg,
	}
}

func (g *GithubOpenConnect) GetConfig() *oauth2.Config {
	return g.cfg
}

func (g *GithubOpenConnect) GetUserName(token *oauth2.Token) string {
	client := githubClient.NewClient(g.cfg.Client(context.TODO(), token))
	user, _, err := client.Users.Get(context.TODO(), "")
	if err != nil {
		utils.Logger.Error("error while retriving user details in github", zap.String("err_msg", err.Error()))
		return ""
	}
	return utils.String(user.Login)
}

func GetGithubUserEmail(token oauth2.TokenSource) string {
	client := githubClient.NewClient(oauth2.NewClient(context.Background(), token))
	user, _, err := client.Users.Get(context.TODO(), "")
	if err != nil {
		utils.Logger.Error("error while retriving user details in github", zap.String("err_msg", err.Error()))
		return ""
	}
	return utils.String(user.Login)
}
