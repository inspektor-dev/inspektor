package types

import (
	"errors"
	"inspektor/utils"
	"net/http"

	"github.com/golang-jwt/jwt"
)

var (
	ErrRoleAlreadyExist = errors.New("role already exist")
	ErrSessionExist     = errors.New("only one session can exist")
	ErrNotExist         = errors.New("items not exist")
)

const (
	ErrInvalidAccess = iota
)

type Claim struct {
	UserName string
	Roles    []string
	ObjectID uint
	jwt.StandardClaims
}

type Ctx struct {
	Rw    http.ResponseWriter
	R     *http.Request
	Claim *Claim
}

type CreateDataSourceRequest struct {
	Name            string   `json:"name"`
	Type            string   `json:"type"`
	Roles           []string `json:"roles"`
	SideCarHostName string   `json:"sidecarHostname"`
}

type CreateUserRequest struct {
	UserName string   `json:"username"`
	Password string   `json:"password"`
	Roles    []string `json:"roles"`
}

func (c *CreateUserRequest) Validate() error {
	if c.UserName == "" {
		return errors.New("username is a required field")
	}
	if c.Password == "" {
		return errors.New("password is a required field")
	}
	if len(c.Roles) == 0 {
		return errors.New("user need atlease one role")
	}
	return nil
}

var ValidDataSources = []string{"postgres"}

func (c *CreateDataSourceRequest) Validate() error {
	if utils.IndexOf(ValidDataSources, c.Type) == -1 {
		return errors.New("not valid data source type")
	}
	if c.Name == "" {
		return errors.New("data souce name can't ne empty")
	}
	if c.SideCarHostName == "" {
		return errors.New("side car hostname can't be nil")
	}
	return nil
}

type CreateSessionRequest struct {
	DatasourceID uint `json:"datasourceId"`
}

type ConfigResponse struct {
	PolicyRepoURL string `json:"policyRepoUrl"`
	PolicyHash    string `json:"policyHash"`
}

type AddRoleRequest struct {
	Type  string   `json:"type"`
	ID    uint     `json:"id"`
	Roles []string `json:"roles"`
}

type OauthResponse struct {
	Provider string `json:"provider"`
	Url      string `json:"url"`
}
