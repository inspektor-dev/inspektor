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
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"inspektor/store"
	"inspektor/utils"
	"net/http"
	"strconv"
	"strings"
	"sync"

	"github.com/google/uuid"
	"github.com/infracloudio/msbotbuilder-go/core"
	"github.com/infracloudio/msbotbuilder-go/core/activity"
	coreActivity "github.com/infracloudio/msbotbuilder-go/core/activity"
	"github.com/infracloudio/msbotbuilder-go/schema"
	"go.uber.org/zap"
)

type AccessRequest struct {
	Account  *schema.ChannelAccount        `json:"account"`
	Database uint                          `json:"database"`
	Roles    []string                      `json:"roles"`
	UserRef  *schema.ConversationReference `json:"userRef"`
}
type TeamsBotHandler struct {
	adapter         core.Adapter
	configToken     string
	adminRef        *schema.ConversationReference
	pendingApproval map[string]AccessRequest
	store           *store.Store
	sentRequestIDs  map[string]interface{}
	sync.Mutex
}

const configureCommand = "configure:"

func New(appID string, password string, store *store.Store) (*TeamsBotHandler, error) {
	setting := core.AdapterSetting{
		AppID:       appID,
		AppPassword: password,
	}

	adapter, err := core.NewBotAdapter(setting)
	if err != nil {
		return nil, err
	}
	return &TeamsBotHandler{
		adapter:         adapter,
		configToken:     utils.GenerateSecureToken(7),
		store:           store,
		pendingApproval: map[string]AccessRequest{},
		sentRequestIDs:  map[string]interface{}{},
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
			// check whether incoming message is configuration message. if it's; configuration
			// message then mark the incoming user as admin user.
			if strings.HasPrefix(turn.Activity.Text, configureCommand) {
				if t.adminRef != nil {
					return turn.SendActivity(activity.MsgOptionText("admin already configured"))
				}
				configToken := strings.Trim(turn.Activity.Text, configureCommand)
				if configToken != t.configToken {
					return turn.SendActivity(activity.MsgOptionText("invalid config tokne"))
				}
				// get the reference of the user if the incoming configuration token is valid
				ref := coreActivity.GetCoversationReference(turn.Activity)
				t.adminRef = &ref
				return turn.SendActivity(activity.MsgOptionText("inspektor is configured for teams"))
			}

			if t.adminRef == nil {
				return turn.SendActivity(activity.MsgOptionText("admin is configure for inspektor bot. Ask your admin to configure"))
			}

			// see incoming request is part of approval response
			t.Lock()
			for requestID := range turn.Activity.Value {
				_, ok := t.sentRequestIDs[requestID]
				if ok {
					delete(t.sentRequestIDs, requestID)
					// it's a approval response so send the approval to admin team
					t.Unlock()
					return t.handleApprovalResponse(requestID, turn)
				}
			}
			t.Unlock()

			// see the incoming request is either approval or deniel flow
			for approvalID, res := range turn.Activity.Value {
				if approvalID == "approval" {
					return t.handleAdminApproval(res.(string), turn)
				}
			}

			requestID := uuid.NewString()
			approvalAttachment, err := t.CreateApprovalView(requestID)
			if err != nil {
				utils.Logger.Error("error while creating approval view on teams bot", zap.String("err_msg", err.Error()))
				return turn.Activity, err
			}
			t.Lock()
			t.sentRequestIDs[requestID] = struct{}{}
			t.Unlock()
			return turn.SendActivity(activity.MsgOptionAttachments(approvalAttachment))
		},
	})
	if err != nil {
		utils.Logger.Error("error while processing user request", zap.String("err_msg", err.Error()))
	}
	utils.WriteSuccesMsg("msg processed", http.StatusOK, w)
}

func (t *TeamsBotHandler) handleAdminApproval(response string, turn *activity.TurnContext) (schema.Activity, error) {
	if strings.Contains(response, "deny") {
		requestID := strings.Trim(response, "deny:")
		t.Lock()
		userRequest, ok := t.pendingApproval[requestID]
		t.Unlock()
		if !ok {
			utils.Logger.Error("unable to find the user request")
			return turn.Activity, errors.New("unable to find user request")
		}
		err := t.adapter.ProactiveMessage(context.TODO(), *userRequest.UserRef, coreActivity.HandlerFuncs{
			OnMessageFunc: func(turn *coreActivity.TurnContext) (schema.Activity, error) {
				return turn.SendActivity(coreActivity.MsgOptionText("your request has been denied by the admin"))
			},
		})
		if err != nil {
			utils.Logger.Error("error while sending deniel message to the user", zap.String("err_msg", err.Error()))
			return turn.Activity, err
		}
	}
	requestID := strings.Trim(response, "aprove:")
	t.Lock()
	userRequest, ok := t.pendingApproval[requestID]
	delete(t.pendingApproval, requestID)
	t.Unlock()
	if !ok {
		utils.Logger.Error("unable to find the user request")
		return turn.Activity, errors.New("unable to find user request")
	}
	session, err := t.store.CreateTempSession(userRequest.Database, userRequest.Roles, 10, "teams", utils.MarshalJSON(userRequest.Account))
	if err != nil {
		utils.Logger.Error("error while creating temp credentials", zap.String("err_msg", err.Error()))
		return turn.Activity, errors.New("error while creating temp crendentials")
	}
	datasource, err := t.store.GetDatasource(userRequest.Database)
	if err != nil {
		utils.Logger.Error("error while retirivg database on approval", zap.String("err_msg", err.Error()))
		return turn.Activity, err
	}
	err = t.adapter.ProactiveMessage(context.TODO(), *userRequest.UserRef, coreActivity.HandlerFuncs{
		OnMessageFunc: func(turn *coreActivity.TurnContext) (schema.Activity, error) {
			return turn.SendActivity(coreActivity.MsgOptionText(fmt.Sprintf(
				`
you request for %s has been approved. 
Use the following credentials
username: %s
password: %s
			`, datasource.Name, session.SessionMeta.PostgresUsername, session.SessionMeta.PostgresPassword)))
		},
	})
	if err != nil {
		utils.Logger.Error("error while sending session data to user", zap.String("err_msg", err.Error()))
		return turn.Activity, err
	}
	return turn.SendActivity(activity.MsgOptionText("your response sent to the user"))
}

func (t *TeamsBotHandler) handleApprovalResponse(requestID string, turn *activity.TurnContext) (schema.Activity, error) {
	dataSourceID, ok := turn.Activity.Value[requestID].(string)
	if !ok {
		utils.Logger.Error("error while finding the datasource id")
		return turn.Activity, errors.New("error while finding datasource id")
	}
	castedDatasourceID, err := strconv.Atoi(dataSourceID)
	if err != nil {
		utils.Logger.Error("error while casting datasource id", zap.String("err_msg", err.Error()))
		return turn.Activity, err
	}
	role, ok := turn.Activity.Value["roles"].(string)
	if !ok {
		utils.Logger.Error("unable to find roles for the given request approval")
		return turn.Activity, errors.New("unable to find roles")
	}
	roles := strings.Split(role, ",")
	t.Lock()
	userRef := coreActivity.GetCoversationReference(turn.Activity)
	request := AccessRequest{
		Account:  &turn.Activity.From,
		UserRef:  &userRef,
		Database: uint(castedDatasourceID),
		Roles:    roles,
	}
	t.pendingApproval[requestID] = request
	t.Unlock()
	adminApprovalView, err := t.CreateApprovalAdminView(requestID, request)
	if err != nil {
		utils.Logger.Error("error while creating admin approval view", zap.String("err_msg", err.Error()))
		return turn.Activity, err
	}
	// send the approval to admin
	err = t.adapter.ProactiveMessage(context.TODO(), *t.adminRef, coreActivity.HandlerFuncs{
		OnMessageFunc: func(turn *coreActivity.TurnContext) (schema.Activity, error) {
			return turn.SendActivity(coreActivity.MsgOptionAttachments(adminApprovalView))
		},
	})
	if err != nil {
		utils.Logger.Error("error while sending approval view to admin", zap.String("err_msg", err.Error()))
		return turn.Activity, err
	}
	return turn.SendActivity(activity.MsgOptionText("Your request is sent for approval to the admin"))
}

func (t *TeamsBotHandler) CreateApprovalView(requestID string) ([]schema.Attachment, error) {
	datasourceNames, datasourceIDs, err := t.store.GetDataSourceWithIDs()
	if err != nil {
		return []schema.Attachment{}, err
	}
	datasourceOptionsView := createOptionsView(datasourceNames, datasourceIDs)
	roles, err := t.store.GetRoles()
	if err != nil {
		return []schema.Attachment{}, err
	}
	rolesOptionsView := createOptionsView(roles, roles)
	// create a raw json view of the approval flow.
	rawJson := fmt.Sprintf(`
	{
		"$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
		"type": "AdaptiveCard",
		"version": "1.0",
		"body": [
		  {
			"type": "Input.ChoiceSet",
			"id": "%s",
			"style": "compact",
			"label": "Select the database you want to access",
			"isMultiSelect": false,
			"value": "1",
			"choices": %s
		  },
		  {
			"type": "Input.ChoiceSet",
			"id": "roles",
			"style": "compact",
			"label": "Select the roles you want",
			"isMultiSelect": true,
			"value": "1",
			"choices": %s
		  }
		],
		"actions": [
		  {
			"type": "Action.Submit",
			"title": "OK"
		  }
		]
	  }
	  `, requestID, datasourceOptionsView, rolesOptionsView)
	// create the teams understandable attachment struct.
	var obj map[string]interface{}
	err = json.Unmarshal(([]byte(rawJson)), &obj)
	if err != nil {
		return []schema.Attachment{}, err
	}
	attachments := []schema.Attachment{
		{
			ContentType: "application/vnd.microsoft.card.adaptive",
			Content:     obj,
		},
	}
	return attachments, nil
}

func createOptionsView(names []string, values []string) string {
	buf := &bytes.Buffer{}
	buf.WriteRune('[')
	for i := 0; i < len(names); i++ {
		buf.WriteRune('{')
		buf.WriteString(fmt.Sprintf(`"title":"%s",`, names[i]))
		buf.WriteString(fmt.Sprintf(`"value":"%s"`, values[i]))
		buf.WriteRune('}')
		if i != len(names)-1 {
			buf.WriteRune(',')
		}
	}

	buf.WriteRune(']')
	return buf.String()
}

func (t *TeamsBotHandler) IsConfigured() bool {
	return t.adminRef != nil
}

func (t *TeamsBotHandler) JoinToken() string {
	return t.configToken
}

func (t *TeamsBotHandler) CreateApprovalAdminView(requestID string, request AccessRequest) ([]schema.Attachment, error) {
	datasource, err := t.store.GetDatasource(request.Database)
	if err != nil {
		utils.Logger.Error("error while retriving datasources", zap.String("err_msg", err.Error()))
		return []schema.Attachment{}, err
	}
	optionsView := createOptionsView([]string{"approve", "deny"}, []string{fmt.Sprintf("aprove:%s", requestID), fmt.Sprintf("deny:%s", requestID)})
	rawJson := fmt.Sprintf(`
	{
		"$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
		"type": "AdaptiveCard",
		"version": "1.0",
		"body": [
		  {
			"type": "TextBlock",
			"size": "Medium",
            "weight": "Bolder",
            "text": "%s has requested to access %s \n with roles %s"
		  },
		  {
			"type": "Input.ChoiceSet",
			"id": "approval",
			"style": "compact",
			"label": "Select the roles you want",
			"isMultiSelect": true,
			"value": "1",
			"choices": %s
		  }
		],
		"actions": [
		  {
			"type": "Action.Submit",
			"title": "OK"
		  }
		]
	  }
	  `, request.Account.Name, datasource.Name, strings.Join(request.Roles, ","), optionsView)
	var obj map[string]interface{}
	err = json.Unmarshal(([]byte(rawJson)), &obj)
	if err != nil {
		return []schema.Attachment{}, err
	}
	attachments := []schema.Attachment{
		{
			ContentType: "application/vnd.microsoft.card.adaptive",
			Content:     obj,
		},
	}
	return attachments, nil
}
