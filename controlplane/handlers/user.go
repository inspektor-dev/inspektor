package handlers

import (
	"encoding/json"
	"inspektor/config"
	"inspektor/store"
	"inspektor/types"
	"inspektor/utils"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt"
	"github.com/gorilla/mux"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type Handlers struct {
	Store *store.Store
	Cfg   *config.Config
}

type LoginRequest struct {
	UserName string `json:"username"`
	Password string `json:"password"`
}

func (h *Handlers) Login() http.HandlerFunc {
	return func(rw http.ResponseWriter, r *http.Request) {
		req := &LoginRequest{}
		if err := json.NewDecoder(r.Body).Decode(req); err != nil {
			utils.WriteErrorMsg("invalid json input", http.StatusBadRequest, rw)
			return
		}
		user, err := h.Store.GetUserByName(req.UserName)
		if err != nil {
			if err == gorm.ErrRecordNotFound {
				utils.WriteErrorMsg("invalid username", http.StatusBadRequest, rw)
				return
			}
			utils.Logger.Error("error while fetch user data", zap.String("err_msg", err.Error()))
			utils.WriteErrorMsg("server down", http.StatusBadRequest, rw)
			return
		}
		if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(req.Password)); err != nil {
			utils.WriteErrorMsg("invalid password", http.StatusBadRequest, rw)
			return
		}
		claim := &types.Claim{
			UserName: req.UserName,
			ObjectID: user.ID,
			StandardClaims: jwt.StandardClaims{
				ExpiresAt: time.Now().Add(time.Hour * 2).Unix(),
			},
		}
		token := jwt.NewWithClaims(jwt.SigningMethodHS256, claim)
		tokenString, err := token.SignedString([]byte(h.Cfg.JwtKey))
		if err != nil {
			utils.Logger.Error("Failed while signing jwt key", zap.String("error_msg", err.Error()))
			utils.WriteErrorMsg("Error while signing key", http.StatusInternalServerError, rw)
			return
		}
		utils.WriteSuccesMsgWithData("", http.StatusOK, struct {
			Token string `json:"token"`
		}{
			Token: tokenString,
		}, rw)
	}
}

func (h *Handlers) Init(router *mux.Router) {
	router.HandleFunc("/login", h.Login()).Methods("POST")
	router.HandleFunc("/datasource", h.AuthMiddleWare(h.CreateDataSource())).Methods("POST")
	router.HandleFunc("/datasource", h.AuthMiddleWare(h.GetDataSources())).Methods("GET")
}
