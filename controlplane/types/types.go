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

var (
	IntegrationConfigKey = "external-integration"
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
	Passthrough  bool `json:"passthrough"`
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

type CreateTempSession struct {
	UserID       uint     `json:"userId"`
	DatasourceID uint     `json:"datasourceId"`
	ExpiryMinute int64    `json:"expiryMinute"`
	Roles        []string `json:"roles"`
}

type IntegrationConfig struct {
	CloudWatchConfig *CloudWatchConfig `json:"cloudWatchConfig"`
	AuditLogConfig   *AuditLogConfig   `json:"auditLogConfig"`
}

type AuditLogConfig struct {
	LogPrefix string `json:"logPrefix"`
}

func (a *AuditLogConfig) Validate() error {
	if a.LogPrefix == "" {
		return errors.New("log prefix is a required to setup stdout audit log")
	}
	return nil
}

type CloudWatchConfig struct {
	CredType      string `json:"credType"`
	RegionName    string `json:"regionName"`
	AccessKey     string `json:"accessKey"`
	SecretKey     string `json:"secretKey"`
	LogGroupName  string `json:"logGroupName"`
	LogStreamName string `json:"logStreamName"`
}

func (s *CloudWatchConfig) Validate() error {
	if utils.IndexOf([]string{"env", "cred"}, s.CredType) == -1 {
		return errors.New("env and cred type is accepted as credential type")
	}
	if s.RegionName == "" {
		return errors.New("region name is mandatory field")
	}
	if s.CredType == "cred" && (s.AccessKey == "" || s.SecretKey == "") {
		return errors.New("access key and secret key is mandatory for the cred type")
	}
	if s.LogGroupName == "" || s.LogStreamName == "" {
		return errors.New("log group name and log stream name are required field")
	}
	return nil
}
