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

	"github.com/slack-go/slack"
	"github.com/slack-go/slack/slackevents"
	"github.com/slack-go/slack/socketmode"
	"go.uber.org/zap"
)

type SlackBot struct {
	client *slack.Client
}

func New(cfg *config.Config) *SlackBot {
	client := slack.New(cfg.SlackBotToken,
		slack.OptionAppLevelToken(cfg.SlackAppToken), slack.OptionDebug(true))
	return &SlackBot{
		client: client,
	}
}

func (s *SlackBot) Start() {
	socketClient := socketmode.New(s.client, socketmode.OptionDebug(true))
	go func() {
		fmt.Println("socket listen")
		for event := range socketClient.Events {
			switch event.Type {
			case socketmode.EventTypeEventsAPI:
				event, ok := event.Data.(*slackevents.EventsAPIEvent)
				if !ok {
					utils.Logger.Error("unexpected slack event api event")
					continue
				}
				s.handleEventApi(event.Data)
			default:
				utils.Logger.Debug("skipping slack event", zap.String("event_name", string(event.Type)))
			}
		}
	}()
	socketClient.Run()
}

func (s *SlackBot) handleEventApi(data interface{}) {
	switch event := data.(type) {
	case *slackevents.AppHomeOpenedEvent:
		fmt.Println("publishing vent")
		res, err := s.client.PublishView(event.User, slack.HomeTabViewRequest{
			Type: slack.VTHomeTab,
			Blocks: slack.Blocks{[]slack.Block{
				slack.ActionBlock{
					Elements: &slack.BlockElements{
						[]slack.BlockElement{
							slack.ButtonBlockElement{
								Type: slack.METButton,
								Text: &slack.TextBlockObject{
									Type: slack.PlainTextType,
									Text: "Request Access",
								},
							},
						},
					},
				},
			}},
			PrivateMetadata: "",
			CallbackID:      "",
			ExternalID:      "",
		}, "")
		if err != nil {
			utils.Logger.Error("error whule publishing view", zap.String("err_msg", err.Error()))
			return
		}
		utils.Logger.Debug("view response", zap.String("res", string(utils.MarshalJSON(res))))
	}
}
