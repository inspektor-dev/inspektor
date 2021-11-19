package types

import (
	"errors"
	"inspektor/utils"
	"net/http"

	"github.com/golang-jwt/jwt"
)

var (
	ErrRoleAlreadyExist = errors.New("role already exist")
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
	Name string `json:"name"`
	Type string `json:"type"`
}

var ValidDataSources = []string{"postgres"}

func (c *CreateDataSourceRequest) Validate() error {
	if utils.IndexOf(ValidDataSources, c.Type) == -1 {
		return errors.New("not valid data source type")
	}
	if c.Name == "" {
		return errors.New("data souce name can't ne empty")
	}
	return nil
}
