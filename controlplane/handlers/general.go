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
	"fmt"
	"inspektor/teamsbot"
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

func (h *Handlers) IntegrationMeta() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsg("invalid access", http.StatusBadRequest, ctx.Rw)
			return
		}
		integrationConfig, err := h.Store.GetIntegrationConfig()
		if err != nil {
			utils.Logger.Error("error while retriving integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		meta := integrationConfig.GetIntegrationMeta()
		h.Lock()
		if meta.IsTeamConfigure && h.teamsBot != nil {
			meta.IsTeamAdminJoined = h.teamsBot.IsConfigured()
			if !meta.IsTeamAdminJoined {
				meta.TeamsJoinToken = h.teamsBot.JoinToken()
			}
		}
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, meta, ctx.Rw)
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
		err := h.UpdateIntegrationCfg(func(cfg *types.IntegrationConfig) {
			cfg.CloudWatchConfig = config
		})
		if err != nil {
			utils.Logger.Error("eror while updating integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}

func (h *Handlers) UpdateIntegrationCfg(cb func(cfg *types.IntegrationConfig)) error {
	val, err := h.Store.Get(types.IntegrationConfigKey)
	if err != nil {
		return fmt.Errorf("error while retriving integration config %s", err.Error())
	}
	integrationConfig := &types.IntegrationConfig{}
	err = json.Unmarshal([]byte(val), integrationConfig)
	if err != nil {
		utils.Logger.Error("error while unmarshaling integration config", zap.String("err_msg", err.Error()))
		return fmt.Errorf("error while unmarshalling integration config %s", err.Error())
	}
	cb(integrationConfig)
	err = h.Store.Update(types.IntegrationConfigKey, string(utils.MarshalJSON(integrationConfig)))
	if err != nil {
		return fmt.Errorf("error while updating config %s", err.Error())
	}
	return nil
}

func (h *Handlers) ConfigureAuditLog() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsg("invalid access", http.StatusBadRequest, ctx.Rw)
			return
		}
		config := &types.AuditLogConfig{}
		if err := json.NewDecoder(ctx.R.Body).Decode(config); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		if err := config.Validate(); err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		err := h.UpdateIntegrationCfg(func(cfg *types.IntegrationConfig) {
			cfg.AuditLogConfig = config
		})
		if err != nil {
			utils.Logger.Error("eror while updating integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}

func (h *Handlers) ConfigureTeams() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsg("invalid access", http.StatusBadRequest, ctx.Rw)
			return
		}
		config := &types.TeamsConfig{}
		if err := json.NewDecoder(ctx.R.Body).Decode(config); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		if err := config.Validate(); err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		bot, err := teamsbot.New(config.AppID, config.AppToken, h.Store)
		if err != nil {
			utils.Logger.Error("error while creating teams bot", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("invalid token", http.StatusBadRequest, ctx.Rw)
			return
		}
		err = h.UpdateIntegrationCfg(func(cfg *types.IntegrationConfig) {
			cfg.TeamsConfig = config
		})
		if err != nil {
			utils.Logger.Error("eror while updating integration config", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		h.Lock()
		h.teamsBot = bot
		h.Unlock()
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}

func (h *Handlers) HandleTeamsMsg() http.HandlerFunc {
	return func(rw http.ResponseWriter, r *http.Request) {
		h.Lock()
		defer h.Unlock()
		if h.teamsBot == nil {
			utils.WriteErrorMsg("teams not configured", http.StatusBadRequest, rw)
			return
		}
		h.teamsBot.HandleTeamsNotification(rw, r)
	}
}
