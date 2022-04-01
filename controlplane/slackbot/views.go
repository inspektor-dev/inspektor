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
	"strings"

	"github.com/slack-go/slack"
)

func NewRequestAccessModal(requestID string, databases []*slack.OptionBlockObject,
	roles []*slack.OptionBlockObject) slack.ModalViewRequest {
	return slack.ModalViewRequest{
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
		CallbackID: requestID,
		Blocks: slack.Blocks{
			BlockSet: []slack.Block{
				slack.InputBlock{
					BlockID: "databaseblock",
					Type:    slack.MBTInput,
					Element: slack.SelectBlockElement{
						Type:        slack.OptTypeStatic,
						Placeholder: slack.NewTextBlockObject(slack.PlainTextType, "slect database", false, false),
						Options:     databases,
						ActionID:    "database",
					},
					Label: &slack.TextBlockObject{
						Type: slack.PlainTextType,
						Text: "Selete database",
					},
				},
				slack.InputBlock{
					BlockID: "rolesblock",
					Type:    slack.MBTInput,
					Element: slack.SelectBlockElement{
						Type:        slack.MultiOptTypeStatic,
						Placeholder: slack.NewTextBlockObject(slack.PlainTextType, "slect database", false, false),
						Options:     roles,
						ActionID:    "roles",
					},
					Label: &slack.TextBlockObject{
						Type: slack.PlainTextType,
						Text: "Selete roles",
					},
				},
			},
		},
	}
}

func NewRequestSentView() slack.HomeTabViewRequest {
	return slack.HomeTabViewRequest{
		Type: slack.VTHomeTab,
		Blocks: slack.Blocks{
			BlockSet: []slack.Block{
				slack.SectionBlock{
					Type: slack.MBTSection,
					Text: slack.NewTextBlockObject(slack.PlainTextType, "Your request have sent to admin for approval✌️", false, false),
				},
			},
		},
	}
}

func NewBlockOptions(options []string, values []string) []*slack.OptionBlockObject {
	opts := []*slack.OptionBlockObject{}
	for i, option := range options {
		opts = append(opts, &slack.OptionBlockObject{
			Text:  slack.NewTextBlockObject(slack.PlainTextType, option, false, false),
			Value: values[i],
		})
	}
	return opts
}

func NewHomeTabView() slack.HomeTabViewRequest {
	return slack.HomeTabViewRequest{
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
	}
}

// credentials approved block
// {
// 	"blocks": [
// 		{
// 			"type": "section",
// 			"text": {
// 				"type": "mrkdwn",
// 				"text": "Your access have approved. credentials can be found below"
// 			}
// 		},
// 		{
// 			"type": "section",
// 			"fields": [
// 				{
// 					"type": "mrkdwn",
// 					"text": "*Username:*\n postgresprod"
// 				},
// 				{
// 					"type": "mrkdwn",
// 					"text": "*Password:*\nadmin, deve, support"
// 				},
// 				{
// 					"type": "mrkdwn",
// 					"text": "*HostName:*\nhttps://github.com"
// 				}
// 			]
// 		}
// 	]
// }

func NewAccessApprovedView() {}

func NewRequestAccessMsg(userName string, db string, roles []string) []slack.Block {
	return []slack.Block{
		slack.SectionBlock{
			Type: slack.MBTSection,
			Text: slack.NewTextBlockObject(slack.PlainTextType, fmt.Sprintf("%s have requested access", userName), false, false),
		},
		slack.SectionBlock{
			Type: slack.MBTSection,
			Fields: []*slack.TextBlockObject{
				slack.NewTextBlockObject(slack.MarkdownType, fmt.Sprintf("*Database:*\n %s", db), false, false),
				slack.NewTextBlockObject(slack.MarkdownType, fmt.Sprintf("*Roles:*\n %s", strings.Join(roles, ",")), false, false),
			},
		},
		slack.ActionBlock{
			Type: slack.MBTAction,
			Elements: &slack.BlockElements{
				ElementSet: []slack.BlockElement{
					slack.ButtonBlockElement{
						Type: slack.METButton,
						Text: &slack.TextBlockObject{
							Type: slack.PlainTextType,
							Text: "Approve",
						},
						Style: "primary",
						Value: "approved",
					},
					slack.ButtonBlockElement{
						Type: slack.METButton,
						Text: &slack.TextBlockObject{
							Type: slack.PlainTextType,
							Text: "Deny",
						},
						Style: "danger",
						Value: "denied",
					},
				},
			},
		},
	}
}

type ModalSubmission struct {
	Values Values `json:"values"`
}

type Text struct {
	Type  string `json:"type"`
	Text  string `json:"text"`
	Emoji bool   `json:"emoji"`
}

type SelectedOption struct {
	Text  Text   `json:"text"`
	Value string `json:"value"`
}

type Database struct {
	Type           string         `json:"type"`
	SelectedOption SelectedOption `json:"selected_option"`
}

type Databaseid struct {
	Database Database `json:"database"`
}

type SelectedOptions struct {
	Text  Text   `json:"text"`
	Value string `json:"value"`
}

type Roles struct {
	Type            string            `json:"type"`
	SelectedOptions []SelectedOptions `json:"selected_options"`
}

type Rolesid struct {
	Roles Roles `json:"roles"`
}

type Values struct {
	Databaseid Databaseid `json:"databaseblock"`
	Rolesid    Rolesid    `json:"rolesblock"`
}
