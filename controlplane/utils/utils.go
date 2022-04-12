package utils

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"inspektor/config"
	"net/http"
	"os"
	"strings"

	"go.uber.org/zap"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

const (
	TokenSize = 30
)

var Logger *zap.Logger

func init() {
	var err error
	if os.Getenv("LOG") == "debug" {
		Logger, err = zap.NewDevelopment()
		Check(err)
	} else {
		Logger, err = zap.NewProduction()
		Check(err)
	}
}

func Check(err error) {
	if err != nil {
		panic(err.Error())
	}
}

type Response struct {
	Msg       string          `json:"msg"`
	Success   bool            `json:"succes"`
	Data      json.RawMessage `json:"data"`
	ErrorCode int             `json:"errCode"`
}

func WriteErrorMsg(msg string, status int, rw http.ResponseWriter) {
	rw.Header().Set("Content-Type", "application/json")
	rw.Header().Set("X-Content-Type-Options", "nosniff")
	rw.Header().Set("Cache-Control", "no-store")
	rw.WriteHeader(status)
	res := &Response{
		Msg:     msg,
		Success: false,
	}
	rw.Write(MarshalJSON(res))
}

func WriteErrorMsgWithErrCode(msg string, code, status int, rw http.ResponseWriter) {
	rw.Header().Set("Content-Type", "application/json")
	rw.Header().Set("X-Content-Type-Options", "nosniff")
	rw.Header().Set("Cache-Control", "no-store")
	rw.WriteHeader(status)
	res := &Response{
		Msg:       msg,
		Success:   false,
		ErrorCode: code,
	}
	rw.Write(MarshalJSON(res))
}

func WriteSuccesMsg(msg string, status int, rw http.ResponseWriter) {
	rw.Header().Set("Content-Type", "application/json")
	rw.Header().Set("X-Content-Type-Options", "nosniff")
	rw.Header().Set("Cache-Control", "no-store")
	rw.WriteHeader(status)
	res := &Response{
		Msg:     msg,
		Success: true,
	}
	rw.Write(MarshalJSON(res))
}

func WriteSuccesMsgWithData(msg string, status int, data interface{}, rw http.ResponseWriter) {
	rw.Header().Set("Content-Type", "application/json")
	rw.Header().Set("X-Content-Type-Options", "nosniff")
	rw.Header().Set("Cache-Control", "no-store")
	rw.WriteHeader(status)
	res := &Response{
		Msg:     msg,
		Success: true,
		Data:    MarshalJSON(data),
	}
	rw.Write(MarshalJSON(res))
}

func MarshalJSON(data interface{}) []byte {
	buf, err := json.Marshal(data)
	Check(err)
	return buf
}

func GetDB(cfg *config.Config) (*gorm.DB, error) {
	ssl := "disable"
	if cfg.PostgresSSL {
		ssl = "enable"
	}
	postgresURL := fmt.Sprintf("host=%s port=%s user=%s dbname=%s password=%s sslmode=%s",
		cfg.PostgresHost,
		cfg.PostgresPort,
		cfg.PostgresUserName,
		cfg.DatabaseName,
		cfg.PostgresPassword,
		ssl)
	return gorm.Open(postgres.Open(postgresURL), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Silent),
	})
}

// IndexOf returns the index of search string in the given input array.
func IndexOf(input []string, x string) int {
	for i, y := range input {
		if y == x {
			return i
		}
	}
	return -1
}

func CommonItemExist(a []string, b []string) bool {
	for _, val := range a {
		if IndexOf(b, val) >= 0 {
			return true
		}
	}
	return false
}

func GenerateSecureToken(length int) string {
	b := make([]byte, length)
	if _, err := rand.Read(b); err != nil {
		return ""
	}
	return hex.EncodeToString(b)
}

// CleanDir will clean the dir.
// will delete and create a new dir for the given
// path.
func CleanDir(dir string) {
	_, err := os.Stat(dir)
	if os.IsNotExist(err) {
		Check(os.MkdirAll(dir, 0755))
		return
	}
	Check(os.RemoveAll(dir))
	Check(os.MkdirAll(dir, 0755))
}

// String converts pointer string to normal string.
func String(in *string) string {
	if in == nil {
		return ""
	}
	return *in
}

func JoinSet(in map[string]struct{}, sep string) string {
	arr := []string{}
	for key := range in {
		arr = append(arr, key)
	}
	return strings.Join(arr, sep)
}
