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

// NewRequestAccessModal will return request access modal view.
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
						Placeholder: slack.NewTextBlockObject(slack.PlainTextType, "select database", false, false),
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
						Placeholder: slack.NewTextBlockObject(slack.PlainTextType, "select roles", false, false),
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

// NewRequestSentView returns request sent view.
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

func NewMarkdownBlock(text string) []slack.Block {
	return []slack.Block{
		slack.SectionBlock{
			Type: slack.MBTSection,
			Text: slack.NewTextBlockObject(slack.MarkdownType, text, false, false),
		},
	}
}

//NewBlockOptions takes options name and it's respective value as input then it returns
// slack option block object.
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

// NewHomeTabView returns slack home tab view.
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

// NewCredentialsBlock returns credentials block. This view is used to show the username and password
// of the datasource.
func NewCredentialsBlock(username, password, hostname string) []slack.Block {
	return []slack.Block{
		slack.SectionBlock{
			Type: slack.MBTSection,
			Text: slack.NewTextBlockObject(slack.PlainTextType, "Your request has been approved", false, false),
		},
		slack.SectionBlock{
			Type: slack.MBTSection,
			Fields: []*slack.TextBlockObject{
				slack.NewTextBlockObject(slack.MarkdownType, fmt.Sprintf("*Username:*\n %s", username), false, false),
				slack.NewTextBlockObject(slack.MarkdownType, fmt.Sprintf("*Password:*\n %s", password), false, false),
				slack.NewTextBlockObject(slack.MarkdownType, fmt.Sprintf("*Hostname:*\n %s", hostname), false, false),
			},
		},
	}
}

// NewRequestAccessMsg returns approval modal for the admins.
func NewRequestAccessMsg(userName string, db string, roles []string, callbackID string) []slack.Block {
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
						Style:    "primary",
						Value:    callbackID,
						ActionID: "approved",
					},
					slack.ButtonBlockElement{
						Type: slack.METButton,
						Text: &slack.TextBlockObject{
							Type: slack.PlainTextType,
							Text: "Deny",
						},
						Style:    "danger",
						Value:    "denied",
						ActionID: "denied",
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
