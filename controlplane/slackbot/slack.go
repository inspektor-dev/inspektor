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

package slackbot

import (
	"fmt"
	"inspektor/config"
	"inspektor/store"
	"inspektor/utils"
	"strconv"

	"github.com/google/uuid"
	"github.com/slack-go/slack"
	"github.com/slack-go/slack/slackevents"
	"github.com/slack-go/slack/socketmode"
	"go.uber.org/zap"
)

type AccessRequest struct {
	UserID   string
	database uint
	Roles    []string
}
type SlackBot struct {
	client          *slack.Client
	adminChannelID  string
	pendingRequests map[string]AccessRequest
	store           *store.Store
}

func New(cfg *config.Config, store *store.Store) *SlackBot {
	client := slack.New(cfg.SlackBotToken,
		slack.OptionAppLevelToken(cfg.SlackAppToken), slack.OptionDebug(true))
	return &SlackBot{
		client:          client,
		adminChannelID:  cfg.SlackAdminChannelID,
		store:           store,
		pendingRequests: make(map[string]AccessRequest),
	}
}

func (s *SlackBot) Start() {
	socketClient := socketmode.New(s.client, socketmode.OptionDebug(true))
	go func() {
		for event := range socketClient.Events {
			switch event.Type {
			case socketmode.EventTypeEventsAPI:
				event, ok := event.Data.(slackevents.EventsAPIEvent)
				if !ok {
					utils.Logger.Error("unexpected slack event api event")
					continue
				}
				s.handleEventApi(event.InnerEvent)
			case socketmode.EventTypeInteractive:
				interactiveEvent, ok := event.Data.(slack.InteractionCallback)
				if !ok {
					continue
				}
				switch interactiveEvent.Type {
				case slack.InteractionTypeBlockActions:
					for _, cb := range interactiveEvent.ActionCallback.BlockActions {
						switch cb.ActionID {
						case "request-access":
							datasources, err := s.store.GetDataSource()
							if err != nil {
								utils.Logger.Error("error while retriving all the datasources", zap.String("err_msg", err.Error()))
								continue
							}
							databaseNames := []string{}
							databaseIDs := []string{}
							for _, datasource := range datasources {
								databaseNames = append(databaseNames, datasource.Name)
								databaseIDs = append(databaseIDs, fmt.Sprintf("%d", datasource.ID))
							}
							databases := NewBlockOptions(databaseNames, databaseIDs)
							roles, err := s.store.GetRoles()
							if err != nil {
								utils.Logger.Error("error while retriving roles", zap.String("err_msg", err.Error()))
								continue
							}
							rolesBlock := NewBlockOptions(roles, roles)
							modalView := NewRequestAccessModal(uuid.NewString(), databases, rolesBlock)
							fmt.Print("\n\n\n\n")
							fmt.Println(string(utils.MarshalJSON(modalView)))
							_, err = s.client.OpenView(interactiveEvent.TriggerID, modalView)
							if err != nil {
								utils.Logger.Error("error while opening modal view", zap.String("err_msg", err.Error()))
							}
							fmt.Println("modal triggered")
						case "denied":
							socketClient.Ack(*event.Request)
							_, _, err := s.client.PostMessage(s.adminChannelID, slack.MsgOptionText("We'll let them know that the request has been denied", false))
							if err != nil {
								utils.Logger.Error("error while approval message to the admin", zap.String("err_msg", err.Error()))
							}
						case "approved":
							socketClient.Ack(*event.Request)
							accessRequest, ok := s.pendingRequests[interactiveEvent.View.CallbackID]
							if !ok {
								_, _, err := s.client.PostMessage(s.adminChannelID, slack.MsgOptionText("Unable to find the request", false))
								if err != nil {
									utils.Logger.Error("error while approval message to the admin", zap.String("err_msg", err.Error()))
								}
								continue
							}
							// apporve request
							fmt.Println(accessRequest)
							// send credentials.
							_, _, err := s.client.PostMessage(s.adminChannelID, slack.MsgOptionText("Thanks for approving. Credentials will be sent to the user", false))
							if err != nil {
								utils.Logger.Error("error while approval message to the admin", zap.String("err_msg", err.Error()))
							}
						}
					}
				case slack.InteractionTypeViewSubmission:
					fmt.Println("callback id ", interactiveEvent.View.CallbackID)
					fmt.Println("raw state\n\n\n")
					fmt.Println(interactiveEvent.RawState)
					fmt.Printf("%+v", interactiveEvent.View.State.Values)
					databaseState, ok := interactiveEvent.View.State.Values["databaseblock"]["database"]
					if !ok {
						utils.Logger.Error("unable to find databaseblock on view submission")
						continue
					}
					rolesState, ok := interactiveEvent.View.State.Values["rolesblock"]["roles"]
					if !ok {
						utils.Logger.Error("unable to find rolesblock on view submission")
						continue
					}
					socketClient.Ack(*event.Request)
					fmt.Println("selected database", databaseState.SelectedOption.Value)
					selectedDatabase, err := strconv.Atoi(databaseState.SelectedOption.Value)
					if err != nil {
						utils.Logger.Error("error while parsinsg database id", zap.String("err_msg", err.Error()))
						continue
					}
					selectedRoles := []string{}
					for _, option := range rolesState.SelectedOptions {
						selectedRoles = append(selectedRoles, option.Value)
					}
					fmt.Println("selected roles", rolesState.SelectedOptions)
					_, err = s.client.PublishView(interactiveEvent.User.ID, NewRequestSentView(), "")
					if err != nil {
						utils.Logger.Error("error while publishing request sent view", zap.String("err_msg", err.Error()))
					}
					_, _, err = s.client.PostMessage(s.adminChannelID,
						slack.MsgOptionBlocks(NewRequestAccessMsg(interactiveEvent.User.ID, databaseState.SelectedOption.Text.Text, selectedRoles, interactiveEvent.View.CallbackID)...))
					if err != nil {
						utils.Logger.Error("error while sending approval message", zap.String("err_msg", err.Error()))
					}
					s.pendingRequests[interactiveEvent.View.CallbackID] = AccessRequest{
						UserID:   interactiveEvent.User.ID,
						database: uint(selectedDatabase),
						Roles:    selectedRoles,
					}
				}

			default:
				utils.Logger.Debug("skipping slack event", zap.String("event_name", string(event.Type)))
			}
		}
	}()
	socketClient.Run()
}

func (s *SlackBot) handleEventApi(data interface{}) {
	switch event := data.(type) {
	case slackevents.EventsAPIInnerEvent:
		innerEvent, ok := event.Data.(*slackevents.AppHomeOpenedEvent)
		if !ok {
			return
		}
		_, err := s.client.PublishView(innerEvent.User, NewHomeTabView(), "")
		if err != nil {
			utils.Logger.Error("error while publishing view", zap.String("err_msg", err.Error()))
			return
		}
	}
}
