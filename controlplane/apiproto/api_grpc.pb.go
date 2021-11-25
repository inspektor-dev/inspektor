// Code generated by protoc-gen-go-grpc. DO NOT EDIT.

package apiproto

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.32.0 or later.
const _ = grpc.SupportPackageIsVersion7

// InspektorClient is the client API for Inspektor service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type InspektorClient interface {
	Auth(ctx context.Context, in *AuthRequest, opts ...grpc.CallOption) (*AuthResponse, error)
	Policy(ctx context.Context, in *Empty, opts ...grpc.CallOption) (Inspektor_PolicyClient, error)
	GetDataSource(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*DataSourceResponse, error)
}

type inspektorClient struct {
	cc grpc.ClientConnInterface
}

func NewInspektorClient(cc grpc.ClientConnInterface) InspektorClient {
	return &inspektorClient{cc}
}

func (c *inspektorClient) Auth(ctx context.Context, in *AuthRequest, opts ...grpc.CallOption) (*AuthResponse, error) {
	out := new(AuthResponse)
	err := c.cc.Invoke(ctx, "/api.Inspektor/Auth", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *inspektorClient) Policy(ctx context.Context, in *Empty, opts ...grpc.CallOption) (Inspektor_PolicyClient, error) {
	stream, err := c.cc.NewStream(ctx, &Inspektor_ServiceDesc.Streams[0], "/api.Inspektor/Policy", opts...)
	if err != nil {
		return nil, err
	}
	x := &inspektorPolicyClient{stream}
	if err := x.ClientStream.SendMsg(in); err != nil {
		return nil, err
	}
	if err := x.ClientStream.CloseSend(); err != nil {
		return nil, err
	}
	return x, nil
}

type Inspektor_PolicyClient interface {
	Recv() (*InspektorPolicy, error)
	grpc.ClientStream
}

type inspektorPolicyClient struct {
	grpc.ClientStream
}

func (x *inspektorPolicyClient) Recv() (*InspektorPolicy, error) {
	m := new(InspektorPolicy)
	if err := x.ClientStream.RecvMsg(m); err != nil {
		return nil, err
	}
	return m, nil
}

func (c *inspektorClient) GetDataSource(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*DataSourceResponse, error) {
	out := new(DataSourceResponse)
	err := c.cc.Invoke(ctx, "/api.Inspektor/GetDataSource", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// InspektorServer is the server API for Inspektor service.
// All implementations must embed UnimplementedInspektorServer
// for forward compatibility
type InspektorServer interface {
	Auth(context.Context, *AuthRequest) (*AuthResponse, error)
	Policy(*Empty, Inspektor_PolicyServer) error
	GetDataSource(context.Context, *Empty) (*DataSourceResponse, error)
	mustEmbedUnimplementedInspektorServer()
}

// UnimplementedInspektorServer must be embedded to have forward compatible implementations.
type UnimplementedInspektorServer struct {
}

func (UnimplementedInspektorServer) Auth(context.Context, *AuthRequest) (*AuthResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Auth not implemented")
}
func (UnimplementedInspektorServer) Policy(*Empty, Inspektor_PolicyServer) error {
	return status.Errorf(codes.Unimplemented, "method Policy not implemented")
}
func (UnimplementedInspektorServer) GetDataSource(context.Context, *Empty) (*DataSourceResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetDataSource not implemented")
}
func (UnimplementedInspektorServer) mustEmbedUnimplementedInspektorServer() {}

// UnsafeInspektorServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to InspektorServer will
// result in compilation errors.
type UnsafeInspektorServer interface {
	mustEmbedUnimplementedInspektorServer()
}

func RegisterInspektorServer(s grpc.ServiceRegistrar, srv InspektorServer) {
	s.RegisterService(&Inspektor_ServiceDesc, srv)
}

func _Inspektor_Auth_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(AuthRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(InspektorServer).Auth(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/api.Inspektor/Auth",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(InspektorServer).Auth(ctx, req.(*AuthRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Inspektor_Policy_Handler(srv interface{}, stream grpc.ServerStream) error {
	m := new(Empty)
	if err := stream.RecvMsg(m); err != nil {
		return err
	}
	return srv.(InspektorServer).Policy(m, &inspektorPolicyServer{stream})
}

type Inspektor_PolicyServer interface {
	Send(*InspektorPolicy) error
	grpc.ServerStream
}

type inspektorPolicyServer struct {
	grpc.ServerStream
}

func (x *inspektorPolicyServer) Send(m *InspektorPolicy) error {
	return x.ServerStream.SendMsg(m)
}

func _Inspektor_GetDataSource_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(Empty)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(InspektorServer).GetDataSource(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/api.Inspektor/GetDataSource",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(InspektorServer).GetDataSource(ctx, req.(*Empty))
	}
	return interceptor(ctx, in, info, handler)
}

// Inspektor_ServiceDesc is the grpc.ServiceDesc for Inspektor service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var Inspektor_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "api.Inspektor",
	HandlerType: (*InspektorServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Auth",
			Handler:    _Inspektor_Auth_Handler,
		},
		{
			MethodName: "GetDataSource",
			Handler:    _Inspektor_GetDataSource_Handler,
		},
	},
	Streams: []grpc.StreamDesc{
		{
			StreamName:    "Policy",
			Handler:       _Inspektor_Policy_Handler,
			ServerStreams: true,
		},
	},
	Metadata: "api.proto",
}
