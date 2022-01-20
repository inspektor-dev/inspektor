// Copyright 2021 Balaji (rbalajis25@gmail.com)
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
package policy

import (
	"inspektor/config"
	"inspektor/utils"
	"sync"

	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/transport/http"
	"github.com/google/uuid"

	"go.uber.org/zap"
)

type PolicyManager struct {
	config      *config.Config
	fsPath      string
	repo        *git.Repository
	subscribers map[string]chan struct{}
	gitEnabled  bool
	sync.Mutex
}

func NewPolicyManager(cfg *config.Config) *PolicyManager {
	utils.CleanDir(cfg.PolicyPath)
	return &PolicyManager{
		config:      cfg,
		fsPath:      cfg.PolicyPath,
		subscribers: make(map[string]chan struct{}),
		gitEnabled:  false,
	}
}

func (p *PolicyManager) Init() error {
	if p.config.PolicyRepo != "" {
		p.gitEnabled = true
	}
	opt := &git.CloneOptions{
		URL: p.config.PolicyRepo,
	}
	if p.config.GithubAccessToken != "" {
		opt.Auth = &http.BasicAuth{
			Username: "inspektor",
			Password: p.config.GithubAccessToken,
		}
	}
	// clone the policy repo to the fs path.
	repo, err := git.PlainClone(p.fsPath, false, opt)
	if err != nil {
		utils.Logger.Error("error while cloning policy repository", zap.String("err_msg", err.Error()))
		return err
	}
	p.repo = repo
	return nil
}

func (p *PolicyManager) Sync() error {
	utils.Logger.Info("syncing policy from git repository")
	w, err := p.repo.Worktree()
	if err != nil {
		utils.Logger.Error("error while retriving worktree", zap.String("err_msg", err.Error()))
		return err
	}
	opt := &git.PullOptions{
		RemoteName: "origin",
	}

	if p.config.GithubAccessToken != "" {
		opt.Auth = &http.BasicAuth{
			Username: "inspektor",
			Password: p.config.GithubAccessToken,
		}
	}
	err = w.Pull(opt)
	if err != nil {
		utils.Logger.Error("error while pulling the policy repository", zap.String("err_msg", err.Error()))
		return err
	}
	p.notify()
	return nil
}

func (p *PolicyManager) Subscribe() (string, chan struct{}) {
	p.Lock()
	defer p.Unlock()
	id := uuid.New().String()
	p.subscribers[id] = make(chan struct{})
	return id, p.subscribers[id]
}

func (p *PolicyManager) Unsubscribe(id string) {
	p.Lock()
	defer p.Unlock()
	delete(p.subscribers, id)
}

func (p *PolicyManager) GetPolicy() ([]byte, error) {
	if !p.gitEnabled {
		return []byte{}, nil
	}
	return Build(p.fsPath)
}

func (p *PolicyManager) notify() {
	p.Lock()
	defer p.Unlock()
	for id, ch := range p.subscribers {
		select {
		case ch <- struct{}{}:
		default:
			utils.Logger.Error("unable to push the policy notification", zap.String("id", id))
		}
	}
}
