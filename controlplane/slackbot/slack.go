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
	"inspektor/utils"

	"github.com/google/uuid"
	"github.com/slack-go/slack"
	"github.com/slack-go/slack/slackevents"
	"github.com/slack-go/slack/socketmode"
	"go.uber.org/zap"
)

type AccessRequest struct {
	UserID string
}
type SlackBot struct {
	client          *slack.Client
	adminChannelID  string
	pendingRequests map[string]AccessRequest
}

func New(cfg *config.Config) *SlackBot {
	client := slack.New(cfg.SlackBotToken,
		slack.OptionAppLevelToken(cfg.SlackAppToken), slack.OptionDebug(true))
	return &SlackBot{
		client:         client,
		adminChannelID: cfg.SlackAdminChannelID,
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
							databases := NewBlockOptions([]string{"databases"}, []string{"databases"})
							roles := NewBlockOptions([]string{"admin", "dev"}, []string{"admin", "dev"})
							_, err := s.client.OpenView(interactiveEvent.TriggerID,
								NewRequestAccessModal(uuid.NewString(), databases, roles))
							if err != nil {
								utils.Logger.Error("error while opening modal view", zap.String("err_msg", err.Error()))
							}
							fmt.Println("modal triggered")
						default:
							socketClient.Ack(*event.Request)
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
					fmt.Println("selected roles", rolesState.SelectedOptions)
					_, err := s.client.PublishView(interactiveEvent.User.ID, NewRequestSentView(), "")
					if err != nil {
						utils.Logger.Error("error while publishing request sent view", zap.String("err_msg", err.Error()))
					}
					_, _, err = s.client.PostMessage(s.adminChannelID,
						slack.MsgOptionBlocks(NewRequestAccessMsg("ppoami", "postgres", []string{"admin", "dev"})...))
					if err != nil {
						utils.Logger.Error("error while sending approval message", zap.String("err_msg", err.Error()))
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
