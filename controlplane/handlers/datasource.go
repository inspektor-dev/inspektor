package handlers

import (
	"encoding/json"
	"inspektor/models"
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

type InspectorHandler func(ctx *types.Ctx)

func (h *Handlers) CreateDataSource() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsgWithErrCode("only admin can create data source", types.ErrInvalidAccess, http.StatusUnauthorized, ctx.Rw)
			return
		}
		req := &types.CreateDataSourceRequest{}
		if err := json.NewDecoder(ctx.R.Body).Decode(req); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		if err := req.Validate(); err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		err := h.Store.CreateDataSource(&models.DataSource{
			Name: req.Name,
			Type: req.Type,
		})
		if err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		utils.WriteSuccesMsg("data soruce created", http.StatusOK, ctx.Rw)
	}
}
