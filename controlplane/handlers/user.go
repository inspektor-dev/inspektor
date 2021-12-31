package handlers

import (
	"encoding/json"
	"inspektor/config"
	"inspektor/policy"
	"inspektor/store"
	"inspektor/types"
	"inspektor/utils"
	"net/http"
	"time"

	"github.com/golang-jwt/jwt"
	"github.com/gorilla/mux"
	"github.com/gorilla/handlers"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type Handlers struct {
	Store  *store.Store
	Cfg    *config.Config
	Policy *policy.PolicyManager
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

func (h *Handlers) PolicyNotification() http.HandlerFunc {
	return func(rw http.ResponseWriter, r *http.Request) {
		if err := h.Policy.Sync(); err != nil {
			utils.Logger.Error("error while syncing policy notification", zap.String("err_msg", err.Error()))
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, rw)
	}
}

func (h *Handlers) Init(router *mux.Router) {
	router.HandleFunc("/login", h.Login()).Methods("POST","OPTIONS")
	router.HandleFunc("/datasource", h.AuthMiddleWare(h.CreateDataSource())).Methods("POST", "OPTIONS")
	router.HandleFunc("/datasource", h.AuthMiddleWare(h.GetDataSources())).Methods("GET", "OPTIONS")
	router.HandleFunc("/session", h.AuthMiddleWare(h.CreateSession())).Methods("POST", "OPTIONS")
	router.HandleFunc("/session", h.AuthMiddleWare(h.GetSesssion())).Methods("GET", "OPTIONS")
	router.HandleFunc("/policy/nofification", h.PolicyNotification()).Methods("POST", "OPTIONS")
	cors := handlers.CORS(
		handlers.AllowedHeaders([]string{"Content-Type", "Auth-Token"}),
		handlers.AllowedOrigins([]string{"*"}),
		handlers.AllowCredentials(),
		handlers.AllowedMethods([]string{"GET", "HEAD", "POST", "DELETE"}),
	)
	router.Use(cors)
}
