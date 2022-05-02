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

package handlers

import (
	"encoding/json"
	"inspektor/types"
	"inspektor/utils"
	"net/http"

	"go.uber.org/zap"
)

func (h *Handlers) Config() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsg("invalid access", http.StatusBadRequest, ctx.Rw)
			return
		}
		res := &types.ConfigResponse{
			PolicyRepoURL: h.Cfg.PolicyRepo,
			PolicyHash:    h.Policy.GetPolicyHash()[:7],
		}
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, res, ctx.Rw)
	}
}

func (h *Handlers) ConfigureCloudWatch() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsg("invalid access", http.StatusBadRequest, ctx.Rw)
			return
		}
		config := &types.CloudWatchConfig{}
		if err := json.NewDecoder(ctx.R.Body).Decode(config); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		if err := config.Validate(); err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		val, err := h.Store.Get(types.IntegrationConfigKey)
		if err != nil {
			utils.Logger.Error("error while retriving integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("error while retriving integration config", http.StatusInternalServerError, ctx.Rw)
			return
		}
		integrationConfig := &types.IntegrationConfig{}
		err = json.Unmarshal([]byte(val), integrationConfig)
		if err != nil {
			utils.Logger.Error("error while unmarshaling integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		integrationConfig.CloudWatchConfig = config
		err = h.Store.Update(types.IntegrationConfigKey, string(utils.MarshalJSON(integrationConfig)))
		if err != nil {
			utils.Logger.Error("error while updating config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}
