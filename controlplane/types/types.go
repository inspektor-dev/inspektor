package types

import (
	"errors"
	"net/http"

	"github.com/golang-jwt/jwt"
)

var (
	ErrRoleAlreadyExist = errors.New("role already exist")
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
