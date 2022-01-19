package handlers

import (
	"encoding/json"
	"inspektor/config"
	"inspektor/models"
	"inspektor/policy"
	"inspektor/store"
	"inspektor/types"
	"inspektor/utils"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"github.com/golang-jwt/jwt"
	"github.com/gorilla/handlers"
	"github.com/gorilla/mux"
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

func (h *Handlers) AddUser() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsgWithErrCode("only admin can add user", types.ErrInvalidAccess, http.StatusUnauthorized, ctx.Rw)
			return
		}

		req := &types.CreateUserRequest{}
		if err := json.NewDecoder(ctx.R.Body).Decode(req); err != nil {
			utils.WriteErrorMsg("invalid json", http.StatusBadRequest, ctx.Rw)
			return
		}

		if err := req.Validate(); err != nil {
			utils.WriteErrorMsg(err.Error(), http.StatusBadRequest, ctx.Rw)
			return
		}

		user, err := h.Store.CreateUser(req.UserName, req.Password)
		if err != nil {
			utils.Logger.Error("error while creating user", zap.String("err_msg", err.Error()))
			handleErr(err, ctx)
			return
		}
		if err := h.Store.WriteRoleForUserObjectID(user.ID, req.Roles); err != nil {
			utils.Logger.Error("error while adding roles to the user", zap.String("err_msg", err.Error()))
			handleErr(err, ctx)
			return
		}
		utils.WriteSuccesMsg("ok", http.StatusOK, ctx.Rw)
	}
}

func (h *Handlers) GetUsers() InspectorHandler {
	return func(ctx *types.Ctx) {
		if utils.IndexOf(ctx.Claim.Roles, "admin") == -1 {
			utils.WriteErrorMsgWithErrCode("only admin can add user", types.ErrInvalidAccess, http.StatusUnauthorized, ctx.Rw)
			return
		}
		users, err := h.Store.GetUsers()
		if err != nil {
			utils.Logger.Error("error while retriving users", zap.String("err_msg", err.Error()))
			handleErr(err, ctx)
			return
		}
		for _, user := range users {
			roles, err := h.Store.GetRolesForObjectID(user.ID, models.UserType)
			if err != nil {
				utils.Logger.Error("error while retriving roles for the user",
					zap.Uint("user_id", user.ID), zap.String("err_msg", err.Error()))
				handleErr(err, ctx)
				return
			}
			user.Roles = roles
		}
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, users, ctx.Rw)
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

func (h *Handlers) Roles() InspectorHandler {
	return func(ctx *types.Ctx) {
		utils.WriteSuccesMsgWithData("ok", http.StatusOK, ctx.Claim.Roles, ctx.Rw)
	}
}

// spaHandler implements the http.Handler interface, so we can use it
// to respond to HTTP requests. The path to the static directory and
// path to the index file within that static directory are used to
// serve the SPA in the given static directory.
type spaHandler struct {
	staticPath string
	indexPath  string
}

// ServeHTTP inspects the URL path to locate a file within the static dir
// on the SPA handler. If a file is found, it will be served. If not, the
// file located at the index path on the SPA handler will be served. This
// is suitable behavior for serving an SPA (single page application).
func (h spaHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// get the absolute path to prevent directory traversal
	path, err := filepath.Abs(r.URL.Path)
	if err != nil {
		// if we failed to get the absolute path respond with a 400 bad request
		// and stop
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	// prepend the path with the path to the static directory
	path = filepath.Join(h.staticPath, path)

	// check whether a file exists at the given path
	_, err = os.Stat(path)
	if os.IsNotExist(err) {
		// file does not exist, serve index.html
		http.ServeFile(w, r, filepath.Join(h.staticPath, h.indexPath))
		return
	} else if err != nil {
		// if we got an error (that wasn't that the file doesn't exist) stating the
		// file, return a 500 internal server error and stop
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// otherwise, use http.FileServer to serve the static dir
	http.FileServer(http.Dir(h.staticPath)).ServeHTTP(w, r)
}

func (h *Handlers) Init(router *mux.Router) {
	router.HandleFunc("/api/login", h.Login()).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/datasource", h.AuthMiddleWare(h.CreateDataSource())).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/datasource", h.AuthMiddleWare(h.GetDataSources())).Methods("GET", "OPTIONS")
	router.HandleFunc("/api/session", h.AuthMiddleWare(h.CreateSession())).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/session", h.AuthMiddleWare(h.GetSesssion())).Methods("GET", "OPTIONS")
	router.HandleFunc("/api/policy/nofification", h.PolicyNotification()).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/user", h.AuthMiddleWare(h.AddUser())).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/users", h.AuthMiddleWare(h.GetUsers())).Methods("GET", "OPTIONS")
	router.HandleFunc("/api/roles", h.AuthMiddleWare(h.Roles())).Methods("GET", "OPTIONS")
	router.HandleFunc("/readiness", func(rw http.ResponseWriter, r *http.Request) {
		rw.Write([]byte("ok"))
	})
	spa := spaHandler{staticPath: "dashboard/dist", indexPath: "index.html"}
	router.PathPrefix("/").Handler(spa)
	cors := handlers.CORS(
		handlers.AllowedHeaders([]string{"Content-Type", "Auth-Token"}),
		handlers.AllowedOrigins([]string{"*"}),
		handlers.AllowCredentials(),
		handlers.AllowedMethods([]string{"GET", "HEAD", "POST", "DELETE"}),
	)
	router.Use(cors)
}
