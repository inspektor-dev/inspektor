package utils

import (
	"encoding/json"
	"net/http"
	"os"

	"go.uber.org/zap"
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
	Msg     string          `json:"msg"`
	Success bool            `json:"succes"`
	Data    json.RawMessage `json:"data"`
}

func WriteErrorMsg(msg string, status int, rw http.ResponseWriter) {
	res := &Response{
		Msg:     msg,
		Success: false,
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
