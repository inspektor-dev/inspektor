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
			return
		}
		if !tkn.Valid {
			utils.WriteErrorMsg("not valid token", http.StatusBadRequest, rw)
			return
		}
		roles, err := h.Store.GetRolesForObjectID(claim.ObjectID, models.UserType)
		if err != nil {
			utils.Logger.Error("error while getting roles in auth handler", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, rw)
			return
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
			Name:            req.Name,
			Type:            req.Type,
			SideCarToken:    utils.GenerateSecureToken(utils.TokenSize),
			SideCarHostName: req.SideCarHostName,
		}, req.Roles)
		if err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}
		utils.WriteSuccesMsg("data soruce created", http.StatusOK, ctx.Rw)
	}
}

func (h *Handlers) GetDataSources() InspectorHandler {
	return func(ctx *types.Ctx) {
		objectIDS, err := h.Store.GetObjectIDsForRoles(models.DataSourceType, ctx.Claim.Roles)
		if err != nil {
			utils.Logger.Error("error while retriving datasource object ids", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		datasources, err := h.Store.GetDataSources(objectIDS...)
		if err != nil {
			utils.Logger.Error("error while retriving data sources", zap.String("err_msg", err.Error()), zap.Uints("ids", objectIDS))
			utils.WriteErrorMsg("server down", http.StatusInternalServerError, ctx.Rw)
			return
		}
		for idx, datasource := range datasources {
			roles, err := h.Store.GetRolesForObjectID(datasource.ID, models.DataSourceType)
			if err != nil {
				utils.Logger.Error("error while retriving data source roles", zap.String("err_msg", err.Error()))
				continue
			}
			datasource.Roles = roles
			datasources[idx] = datasource
		}
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, datasources, ctx.Rw)
	}
}

func (h *Handlers) DeleteDatasource() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsgWithErrCode("only admin can create data source", types.ErrInvalidAccess, http.StatusUnauthorized, ctx.Rw)
			return
		}
		req := &types.DeleteDatasourceRequest{}
		if err := json.NewDecoder(ctx.R.Body).Decode(req); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}
		h.Store.DeleteDatasource(req.DatasourceID)
		h.Store.DeleteSessionsForDatasource(req.DatasourceID, ctx.Claim.ObjectID)
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}
