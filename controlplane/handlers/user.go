package handlers

import (
	"encoding/json"
	"inspektor/store"
	"inspektor/utils"
	"net/http"

	"github.com/gorilla/mux"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type Handlers struct {
	Store  *store.Store
	Router *mux.Router
}

type LoginRequest struct {
	UserName string `json:"userName"`
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

	}
}
