package utils

import (
	"encoding/json"
	"fmt"
	"inspektor/config"
	"net/http"
	"os"

	"go.uber.org/zap"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
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
	res := &Response{
		Msg:     msg,
		Success: false,
	}
	rw.Write(MarshalJSON(res))
}

func WriteErrorMsgWithErrCode(msg string, code, status int, rw http.ResponseWriter) {
	res := &Response{
		Msg:       msg,
		Success:   false,
		ErrorCode: code,
	}
	rw.Write(MarshalJSON(res))
}

func WriteSuccesMsg(msg string, status int, rw http.ResponseWriter) {
	res := &Response{
		Msg:     msg,
		Success: true,
	}
	rw.Write(MarshalJSON(res))
}

func WriteSuccesMsgWithData(msg string, status int, data interface{}, rw http.ResponseWriter) {
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
