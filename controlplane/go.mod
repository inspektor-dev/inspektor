module inspektor

go 1.16

require (
	github.com/go-git/go-git/v5 v5.4.2
	github.com/golang-jwt/jwt v3.2.2+incompatible
	github.com/google/uuid v1.3.0
	github.com/goombaio/namegenerator v0.0.0-20181006234301-989e774b106e
	github.com/gorilla/handlers v1.5.1
	github.com/gorilla/mux v1.8.0
	github.com/open-policy-agent/opa v0.34.2
	github.com/spf13/cobra v1.2.1
	github.com/spf13/viper v1.8.1
	go.uber.org/zap v1.19.1
	golang.org/x/crypto v0.0.0-20211117183948-ae814b36b871
	google.golang.org/grpc v1.38.0
	google.golang.org/protobuf v1.27.1
	gopkg.in/fsnotify.v1 v1.4.7 // indirect
	gorm.io/datatypes v1.0.3
	gorm.io/driver/postgres v1.2.2
	gorm.io/gorm v1.22.3
)
