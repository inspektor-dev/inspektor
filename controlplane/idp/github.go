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

package idp

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"inspektor/utils"

	"github.com/shurcooL/githubv4"
	"go.uber.org/zap"
	"golang.org/x/oauth2"
)

type GhServiceAccCfg struct {
	UserName            string `json:"username"`
	PersonalAccessToken string `json:"personal_access_token"`
}

type GithubClient struct {
	ghCfg  *GhServiceAccCfg
	client *githubv4.Client
}

func NewGithubClient(serviceToken string) (IdpClient, error) {
	buf, err := base64.StdEncoding.DecodeString(serviceToken)
	if err != nil {
		utils.Logger.Error("error while decoding base66 service token", zap.String("err_msg", err.Error()))
		return nil, err
	}
	cfg := &GhServiceAccCfg{}
	if err := json.Unmarshal(buf, cfg); err != nil {
		utils.Logger.Error("error while unmarshaling github service token", zap.String("err_msg", err.Error()))
		return nil, err
	}
	src := oauth2.StaticTokenSource(
		&oauth2.Token{AccessToken: cfg.PersonalAccessToken},
	)
	httpClient := oauth2.NewClient(context.Background(), src)
	client := githubv4.NewClient(httpClient)

	return &GithubClient{
		ghCfg:  cfg,
		client: client,
	}, err
}

func (g *GithubClient) GetRoles(username string) ([]string, error) {
	var query struct {
		Organization struct {
			Teams struct {
				Edges []struct {
					Node struct {
						Name string
					}
				}
			} `graphql:"teams(first: 20, userLogins: $logins)"`
		} `graphql:"organization(login:$orgname)"`
	}

	variables := map[string]interface{}{
		"orgname": githubv4.String(g.ghCfg.UserName),
		"logins":  []githubv4.String{githubv4.String(username)},
	}
	err := g.client.Query(context.TODO(), &query, variables)
	if err != nil {
		utils.Logger.Error("error while querying organization", zap.String("err_msg", err.Error()))
		return nil, err
	}
	roles := []string{}
	for _, edge := range query.Organization.Teams.Edges {
		roles = append(roles, edge.Node.Name)
	}
	return roles, nil
}
