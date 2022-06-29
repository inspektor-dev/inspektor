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

package teamsbot

import (
	"context"
	"inspektor/utils"
	"net/http"

	"github.com/infracloudio/msbotbuilder-go/core"
	"github.com/infracloudio/msbotbuilder-go/core/activity"
	"github.com/infracloudio/msbotbuilder-go/schema"
	"go.uber.org/zap"
)

type TeamsBotHandler struct {
	adapter core.Adapter
}

func New(appID string, password string) (*TeamsBotHandler, error) {
	setting := core.AdapterSetting{
		AppID:       appID,
		AppPassword: password,
	}

	adapter, err := core.NewBotAdapter(setting)
	if err != nil {
		return nil, err
	}
	return &TeamsBotHandler{
		adapter: adapter,
	}, nil
}

func (t *TeamsBotHandler) HandleTeamsNotification(w http.ResponseWriter, req *http.Request) {
	ctx := context.Background()
	userRequest, err := t.adapter.ParseRequest(ctx, req)
	if err != nil {
		utils.Logger.Error("error while parsing teams request", zap.String("err_msg", err.Error()))
		utils.WriteErrorMsg("server down", http.StatusInternalServerError, w)
		return
	}

	err = t.adapter.ProcessActivity(ctx, userRequest, activity.HandlerFuncs{
		OnMessageFunc: func(turn *activity.TurnContext) (schema.Activity, error) {
			return turn.SendActivity(activity.MsgOptionText("Echo: " + turn.Activity.Text))
		},
	})
	if err != nil {
		utils.Logger.Error("error while processing user request", zap.String("err_msg", err.Error()))
	}
	utils.WriteSuccesMsg("msg processed", http.StatusOK, w)
}
