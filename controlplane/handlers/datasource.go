package handlers

import (
	"inspektor/types"
	"inspektor/utils"
	"net/http"

	"github.com/golang-jwt/jwt"
	"go.uber.org/zap"
)

func (h *Handlers) AuthMiddleWare(next func(ctx *types.Ctx)) http.HandlerFunc {
	return func(rw http.ResponseWriter, r *http.Request) {
		token := r.Header.Get("Auth-Token")
		claim := &types.Claim{}
		tkn, err := jwt.ParseWithClaims(token, claim, func(token *jwt.Token) (interface{}, error) {
			return []byte(h.Cfg.JwtKey), nil
		})
		if err != nil {
			utils.Logger.Error("error while parsing claim", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("bad token", http.StatusBadRequest, rw)
		}
		if !tkn.Valid {
			utils.WriteErrorMsg("not valid token", http.StatusBadRequest, rw)
		}
		roles, err := h.Store.GetRolesForObjectID(claim.ObjectID)
		if err != nil {
			utils.Logger.Error("error while getting roles in auth handler", zap.String("err_msg", err.Error()))
		}
		claim.Roles = roles
		next(&types.Ctx{
			Rw:    rw,
			R:     r,
			Claim: claim,
		})
	}
}
