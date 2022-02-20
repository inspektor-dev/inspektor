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
	"errors"
	"inspektor/config"
	"inspektor/utils"

	"go.uber.org/zap"
)

type IdpClient interface {
	GetRoles(username string) ([]string, error)
}

var idpClients = map[string]func(serviceToken string) (IdpClient, error){"github": NewGithubClient}

func GetIdpClient(cfg *config.Config) (IdpClient, error) {
	utils.Logger.Info("retriving ldp provider")
	constructor, ok := idpClients[cfg.IdpProvider]
	if !ok {
		utils.Logger.Error("error while retriving ldp client provider", zap.String("provider_name", cfg.IdpProvider))
		return nil, errors.New("unable to find ldp provider")
	}
	return constructor(cfg.IdpServiceAccount)
}
