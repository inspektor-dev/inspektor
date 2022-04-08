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
	User     *slack.User
	database uint
	Roles    []string
}
type SlackBot struct {
	client          *slack.Client
	adminChannelID  string
	pendingRequests map[string]AccessRequest
	store           *store.Store
	socketClient    *socketmode.Client
}

// New will return slack bot.
func New(cfg *config.Config, store *store.Store) *SlackBot {
	client := slack.New(cfg.SlackBotToken,
		slack.OptionAppLevelToken(cfg.SlackAppToken), slack.OptionDebug(false))
	return &SlackBot{
		client:          client,
		adminChannelID:  cfg.SlackAdminChannelID,
		store:           store,
		pendingRequests: make(map[string]AccessRequest),
	}
}

// Start will start listening for websocket event and handles all the interative events.
// eg: request submission and credentials delivery.
func (s *SlackBot) Start() {
	s.socketClient = socketmode.New(s.client, socketmode.OptionDebug(false))
	go func() {
		for event := range s.socketClient.Events {
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
						s.socketClient.Ack(*event.Request)
						s.handleAction(cb, interactiveEvent)
					}
				case slack.InteractionTypeViewSubmission:
					s.socketClient.Ack(*event.Request)
					s.handleRequestSubmission(interactiveEvent)
				}

			default:
				utils.Logger.Debug("skipping slack event", zap.String("event_name", string(event.Type)))
			}
		}
	}()
	s.socketClient.Run()
}

// handleEventApi shows entry point for the user to intiate request access.
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

// handleRequestSubmission will handle the form submission event. We do get this event only if
// the user submitted request access modal.
func (s *SlackBot) handleRequestSubmission(iEvent slack.InteractionCallback) {
	// retrive the value of datsource id and roles which they want to gain.
	databaseState, ok := iEvent.View.State.Values["databaseblock"]["database"]
	if !ok {
		utils.Logger.Error("unable to find databaseblock on view submission")
		return
	}
	rolesState, ok := iEvent.View.State.Values["rolesblock"]["roles"]
	if !ok {
		utils.Logger.Error("unable to find rolesblock on view submission")
		return
	}
	selectedDatabase, err := strconv.Atoi(databaseState.SelectedOption.Value)
	if err != nil {
		utils.Logger.Error("error while parsinsg database id", zap.String("err_msg", err.Error()))
		return
	}
	selectedRoles := []string{}
	for _, option := range rolesState.SelectedOptions {
		selectedRoles = append(selectedRoles, option.Value)
	}

	// track the request in the memory, so if the admin approve we can map the approval
	// with the request and can send credentials to the user.
	user, err := s.client.GetUserInfo(iEvent.User.ID)
	if err != nil {
		utils.Logger.Error("error while retriving user details", zap.String("err_msg", err.Error()))
		return
	}
	s.pendingRequests[iEvent.View.CallbackID] = AccessRequest{
		User:     user,
		database: uint(selectedDatabase),
		Roles:    selectedRoles,
	}
	// send approval view for the admin to approve or deny the request.
	_, err = s.client.PublishView(iEvent.User.ID, NewRequestSentView(), "")
	if err != nil {
		utils.Logger.Error("error while publishing request sent view", zap.String("err_msg", err.Error()))
	}
	_, _, err = s.client.PostMessage(s.adminChannelID,
		slack.MsgOptionBlocks(NewRequestAccessMsg(user.Name, databaseState.SelectedOption.Text.Text, selectedRoles, iEvent.View.CallbackID)...))
	if err != nil {
		utils.Logger.Error("error while sending approval message", zap.String("err_msg", err.Error()))
	}

}

func (s *SlackBot) handleAction(action *slack.BlockAction, iEvent slack.InteractionCallback) {
	switch action.ActionID {
	case "request-access":
		// access has been requested. So, open a modal where user can choose
		// the database and roles which they want to gain.
		datasources, err := s.store.GetDataSource()
		if err != nil {
			utils.Logger.Error("error while retriving all the datasources", zap.String("err_msg", err.Error()))
			return
		}
		databaseNames := []string{}
		databaseIDs := []string{}
		for _, datasource := range datasources {
			databaseNames = append(databaseNames, datasource.Name)
			databaseIDs = append(databaseIDs, fmt.Sprintf("%d", datasource.ID))
		}
		roles, err := s.store.GetRoles()
		if err != nil {
			utils.Logger.Error("error while retriving roles", zap.String("err_msg", err.Error()))
			return
		}

		// create the modal view for the user.
		rolesBlock := NewBlockOptions(roles, roles)
		databases := NewBlockOptions(databaseNames, databaseIDs)
		modalView := NewRequestAccessModal(uuid.NewString(), databases, rolesBlock)
		_, err = s.client.OpenView(iEvent.TriggerID, modalView)
		if err != nil {
			utils.Logger.Error("error while opening modal view", zap.String("err_msg", err.Error()))
		}
	case "denied":
		// request has been denied, so let the requested user know that the request has been
		// denied.
		s.postMessage(s.adminChannelID, "We'll let them know that the request has been denied")
		accessRequest, ok := s.pendingRequests[action.Value]
		if !ok {
			return
		}
		s.postMessage(accessRequest.User.ID, "Your access request has been denied")
	case "approved":
		// request has been approved, so create the temp session and send the credentials as direct message
		// to the requested user.
		accessRequest, ok := s.pendingRequests[action.Value]
		if !ok {
			_, _, err := s.client.PostMessage(s.adminChannelID, slack.MsgOptionText("Unable to find the request", false))
			if err != nil {
				utils.Logger.Error("error while approval message to the admin", zap.String("err_msg", err.Error()))
			}
			return
		}
		session, err := s.store.CreateTempSession(accessRequest.database, accessRequest.Roles, 10, "slack", utils.MarshalJSON(accessRequest.User))
		if err != nil {
			utils.Logger.Error("error while creating temp credentials", zap.String("err_msg", err.Error()))
			return
		}
		datasource, err := s.store.GetDatasource(accessRequest.database)
		if err != nil {
			utils.Logger.Error("error while retirving datasource", zap.String("err_msg", err.Error()))
			s.postMessage(s.adminChannelID, "unable to retrive datasoruce information")
			return
		}
		credentialBlock := NewCredentialsBlock(session.SessionMeta.PostgresUsername, session.SessionMeta.PostgresPassword, datasource.SideCarHostName)
		_, _, err = s.client.PostMessage(accessRequest.User.ID, slack.MsgOptionBlocks(credentialBlock...))
		if err != nil {
			utils.Logger.Error("error while sending credentials to the user")
			s.postMessage(s.adminChannelID, "error while sending credentials to the user")
			return
		}
		s.postMessage(s.adminChannelID, "Thanks for approving. Credentials will be sent to the user")
	}
}

// postMessage will send plain text message to the given channel id.
func (s *SlackBot) postMessage(channelID string, msg string) {
	_, _, err := s.client.PostMessage(channelID, slack.MsgOptionText(msg, false))
	if err != nil {
		utils.Logger.Error("error while approval message to the admin", zap.String("err_msg", err.Error()))
	}
}

func (s *SlackBot) PostMarkdownMsg(msg string) error {
	_, _, err := s.client.PostMessage(s.adminChannelID, slack.MsgOptionBlocks(slack.NewTextBlockObject(slack.MarkdownType, msg, false, false)))
	return err
}
