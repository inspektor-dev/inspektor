package rpcserver

import (
	"context"
	"errors"
	"fmt"
	"inspektor/apiproto"
	"inspektor/config"
	"inspektor/models"
	"inspektor/policy"
	"inspektor/store"
	"inspektor/utils"
	"net"
	"time"

	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

type CtxKey string

var DataSource CtxKey = "datasource"

type RpcServer struct {
	store *store.Store
	apiproto.UnimplementedInspektorServer
}

func NewServer(store *store.Store) *RpcServer {
	return &RpcServer{
		store: store,
	}
}

func (r *RpcServer) Auth(ctx context.Context, req *apiproto.AuthRequest) (*apiproto.Empty, error) {
	return nil, nil
}

func (r *RpcServer) Policy(req *apiproto.Empty, stream apiproto.Inspektor_PolicyServer) error {
	byteCode, err := policy.Build("")
	if err != nil {
		utils.Logger.Error("error while building policy", zap.String("err_msg", err.Error()))
		return errors.New("unable to build policy")
	}
	stream.Send(&apiproto.InspektorPolicy{
		WasmByteCode: byteCode,
	})
	time.Sleep(time.Hour)
	return nil
}

func (r *RpcServer) GetDataSource(ctx context.Context, req *apiproto.Empty) (*apiproto.DataSourceResponse, error) {
	dataSource, ok := ctx.Value(DataSource).(*models.DataSource)
	if !ok {
		return nil, errors.New("unable to find the datasource")
	}
	return &apiproto.DataSourceResponse{
		DataSourceName: dataSource.Name,
	}, nil
}

func (r *RpcServer) getAuthInterceptor() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (resp interface{}, err error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return nil, fmt.Errorf("couldn't parse incoming context metadata")
		}
		vals := md.Get("Auth-Token")
		if len(vals) == 0 {
			return nil, fmt.Errorf("auth token header is required")
		}
		fmt.Println("interceptor called")
		token := vals[0]
		dataSource, err := r.store.GetDatasourceByWhere("side_car_token = ?", token)
		if err != nil {
			utils.Logger.Error("error while retriving data source while authentication", zap.String("err_msg", err.Error()))
			return nil, err
		}
		ctx = context.WithValue(ctx, DataSource, dataSource)
		return handler(ctx, req)
	}
}

func (r *RpcServer) getAuthStreamInterceptor() grpc.StreamServerInterceptor {
	return func(srv interface{}, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
		ctx := ss.Context()
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return fmt.Errorf("couldn't parse incoming context metadata")
		}
		vals := md.Get("Auth-Token")
		if len(vals) == 0 {
			return fmt.Errorf("auth token header is required")
		}
		fmt.Println("interceptor called")
		token := vals[0]
		_, err := r.store.GetDatasourceByWhere("side_car_token = ?", token)
		if err != nil {
			utils.Logger.Error("error while retriving data source while authentication", zap.String("err_msg", err.Error()))
			return err
		}
		return handler(srv, ss)
	}
}

func (r *RpcServer) Start(cfg *config.Config) error {
	utils.Logger.Info("grpc server starting", zap.String("port", cfg.GrpcListenPort))
	lis, err := net.Listen("tcp", cfg.GrpcListenPort)
	if err != nil {
		utils.Logger.Error("error while listening port for grpc connections", zap.String("err_msg", err.Error()))
		return err
	}
	server := grpc.NewServer(grpc.UnaryInterceptor(r.getAuthInterceptor()), grpc.StreamInterceptor(r.getAuthStreamInterceptor()))
	apiproto.RegisterInspektorServer(server, r)
	return server.Serve(lis)
}
