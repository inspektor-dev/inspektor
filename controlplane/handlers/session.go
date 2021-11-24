package handlers

import (
	"encoding/json"
	"inspektor/models"
	"inspektor/types"
	"inspektor/utils"
	"net/http"

	"go.uber.org/zap"
)

func (h *Handlers) GetSesssion() InspectorHandler {
	return func(ctx *types.Ctx) {
		sessions, err := h.Store.GetSessionForUser(ctx.Claim.ObjectID)
		if err != nil {
			utils.Logger.Error("error while retriving user session", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, utils.MarshalJSON(sessions), ctx.Rw)
	}
}

func (h *Handlers) CreateSession() InspectorHandler {
	return func(ctx *types.Ctx) {
		req := &types.CreateSessionRequest{}
		if err := json.NewDecoder(ctx.R.Body).Decode(req); err != nil {
			utils.Logger.Error("error while decoding user request")
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		// validate whether user have access to the particular datasource.
		roles, err := h.Store.GetRolesForObjectID(req.DatasourceID, models.DataSourceType)
		if err != nil {
			utils.Logger.Error("error while retriving roles for the datasource id", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}

		haveAccess := utils.CommonItemExist(roles, ctx.Claim.Roles)
		if !haveAccess {
			utils.WriteErrorMsg("unauthorized access", http.StatusBadRequest, ctx.Rw)
			return
		}
		err = h.Store.CreateSessionForUser(ctx.Claim.ObjectID, req.DatasourceID)
		if err != nil {
			handleErr(err, ctx)
			return
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}

func handleErr(err error, ctx *types.Ctx) {
	switch err {
	case types.ErrSessionExist:
		utils.WriteErrorMsg("session already exist for this datasource", http.StatusBadRequest, ctx.Rw)
	default:
		utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
	}
}
