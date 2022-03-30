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
				event, ok := event.Data.(slackevents.EventsAPIEvent)
				if !ok {
					utils.Logger.Error("unexpected slack event api event")
					continue
				}
				s.handleEventApi(event.InnerEvent)
			case socketmode.EventTypeInteractive:
				action, ok := event.Data.(slack.InteractionCallback)
				if !ok {
					continue
				}
				s.handleActionCB(action.ActionCallback, action.TriggerID)
			default:
				utils.Logger.Debug("skipping slack event", zap.String("event_name", string(event.Type)))
			}
		}
	}()
	socketClient.Run()
}

func (s *SlackBot) handleActionCB(cbs slack.ActionCallbacks, triggerID string) {
	for _, cb := range cbs.BlockActions {
		switch cb.ActionID {
		case "request-access":
			_, err := s.client.OpenView(triggerID, slack.ModalViewRequest{
				Type: slack.VTModal,
				Title: &slack.TextBlockObject{
					Type: slack.PlainTextType,
					Text: "Request Temporary Access",
				},
				Submit: &slack.TextBlockObject{
					Type: slack.PlainTextType,
					Text: "Request Access",
				},
				Close: &slack.TextBlockObject{
					Type: slack.PlainTextType,
					Text: "Cancel",
				},
				Blocks: slack.Blocks{
					BlockSet: []slack.Block{
						slack.SectionBlock{
							Type: slack.MBTSection,
							Text: &slack.TextBlockObject{
								Type: slack.PlainTextType,
								Text: "Select database you wish to access",
							},
							Accessory: &slack.Accessory{
								SelectElement: &slack.SelectBlockElement{
									Type: slack.OptTypeStatic,
									Placeholder: &slack.TextBlockObject{
										Type: slack.PlainTextType,
										Text: "Select a Database",
									},
									Options: []*slack.OptionBlockObject{
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "postgres-prod",
											},
											Value: "postgres-prod",
										},
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "postgres-prod1",
											},
											Value: "postgres-prod1",
										},
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "postgres-prod2",
											},
											Value: "postgres-prod2",
										},
									},
								},
							},
						},
						slack.SectionBlock{
							Type: slack.MBTSection,
							Text: &slack.TextBlockObject{
								Type: slack.PlainTextType,
								Text: "Select roles you wish to obtain",
							},
							Accessory: &slack.Accessory{
								SelectElement: &slack.SelectBlockElement{
									Type: slack.MultiOptTypeStatic,
									Placeholder: &slack.TextBlockObject{
										Type: slack.PlainTextType,
										Text: "Select roles",
									},
									Options: []*slack.OptionBlockObject{
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "admin",
											},
											Value: "admin",
										},
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "support",
											},
											Value: "support",
										},
										&slack.OptionBlockObject{
											Text: &slack.TextBlockObject{
												Type: slack.PlainTextType,
												Text: "dev",
											},
											Value: "dev",
										},
									},
								},
							},
						},
					},
				},
			})
			if err != nil {
				utils.Logger.Error("error while opening modal view", zap.String("err_msg", err.Error()))
			}
			fmt.Println("modal triggered")
		}
	}
}

func (s *SlackBot) handleEventApi(data interface{}) {
	fmt.Println("event api\n\n\n\n\n\n\n\n\n\n\n\n\n")
	fmt.Println(string(utils.MarshalJSON(data)))
	switch event := data.(type) {
	case slackevents.EventsAPIInnerEvent:
		innerEvent, ok := event.Data.(*slackevents.AppHomeOpenedEvent)
		if !ok {
			return
		}
		fmt.Println("publishing vent")
		// {
		// 	"blocks": [
		// 		{
		// 			"type": "section",
		// 			"text": {
		// 				"type": "mrkdwn",
		// 				"text": "Hi👋, I'm Inspektor bot 🤖. you can request for \n database access through me"
		// 			},
		// 			"accessory": {
		// 				"type": "button",
		// 				"text": {
		// 					"type": "plain_text",
		// 					"text": "Request Access",
		// 					"emoji": true
		// 				},
		// 				"value": "click_me_123",
		// 				"action_id": "button-action"
		// 			}
		// 		}
		// 	]
		// }
		// slack.ActionBlock{
		// 	Type: slack.MBTAction,
		// 	Elements: &slack.BlockElements{
		// 		[]slack.BlockElement{
		// 			slack.ButtonBlockElement{
		// 				Type: slack.METButton,
		// 				Text: &slack.TextBlockObject{
		// 					Type: slack.PlainTextType,
		// 					Text: "Request Access",
		// 				},
		// 			},
		// 		},
		// 	},
		// },
		res, err := s.client.PublishView(innerEvent.User, slack.HomeTabViewRequest{
			Type: slack.VTHomeTab,
			Blocks: slack.Blocks{[]slack.Block{
				slack.SectionBlock{
					Type: slack.MBTSection,
					Text: &slack.TextBlockObject{
						Type: slack.MarkdownType,
						Text: "Hi👋, I'm Inspektor bot 🤖. you can request for database access through me",
					},
					Accessory: &slack.Accessory{
						ButtonElement: &slack.ButtonBlockElement{
							Type: slack.METButton,
							Text: &slack.TextBlockObject{
								Type:  slack.PlainTextType,
								Text:  "Request Access",
								Emoji: true,
							},
							Value:    "access_requested",
							ActionID: "request-access",
						},
					},
				},
			}},
		}, "")
		if err != nil {
			utils.Logger.Error("error whule publishing view", zap.String("err_msg", err.Error()))
			return
		}
		utils.Logger.Debug("view response", zap.String("res", string(utils.MarshalJSON(res))))
	}
}
