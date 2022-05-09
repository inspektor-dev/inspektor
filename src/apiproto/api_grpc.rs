// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_INSPEKTOR_AUTH: ::grpcio::Method<super::api::AuthRequest, super::api::AuthResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Inspektor/Auth",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_INSPEKTOR_POLICY: ::grpcio::Method<super::api::Empty, super::api::InspektorPolicy> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/api.Inspektor/Policy",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_INSPEKTOR_GET_DATA_SOURCE: ::grpcio::Method<super::api::Empty, super::api::DataSourceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Inspektor/GetDataSource",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_INSPEKTOR_SEND_METRICS: ::grpcio::Method<super::api::MetricsRequest, super::api::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Inspektor/SendMetrics",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_INSPEKTOR_GET_INTEGRATION_CONFIG: ::grpcio::Method<super::api::Empty, super::api::IntegrationConfigResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/api.Inspektor/GetIntegrationConfig",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct InspektorClient {
    client: ::grpcio::Client,
}

impl InspektorClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        InspektorClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn auth_opt(&self, req: &super::api::AuthRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::AuthResponse> {
        self.client.unary_call(&METHOD_INSPEKTOR_AUTH, req, opt)
    }

    pub fn auth(&self, req: &super::api::AuthRequest) -> ::grpcio::Result<super::api::AuthResponse> {
        self.auth_opt(req, ::grpcio::CallOption::default())
    }

    pub fn auth_async_opt(&self, req: &super::api::AuthRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::AuthResponse>> {
        self.client.unary_call_async(&METHOD_INSPEKTOR_AUTH, req, opt)
    }

    pub fn auth_async(&self, req: &super::api::AuthRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::AuthResponse>> {
        self.auth_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn policy_opt(&self, req: &super::api::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::api::InspektorPolicy>> {
        self.client.server_streaming(&METHOD_INSPEKTOR_POLICY, req, opt)
    }

    pub fn policy(&self, req: &super::api::Empty) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::api::InspektorPolicy>> {
        self.policy_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_data_source_opt(&self, req: &super::api::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::DataSourceResponse> {
        self.client.unary_call(&METHOD_INSPEKTOR_GET_DATA_SOURCE, req, opt)
    }

    pub fn get_data_source(&self, req: &super::api::Empty) -> ::grpcio::Result<super::api::DataSourceResponse> {
        self.get_data_source_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_data_source_async_opt(&self, req: &super::api::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::DataSourceResponse>> {
        self.client.unary_call_async(&METHOD_INSPEKTOR_GET_DATA_SOURCE, req, opt)
    }

    pub fn get_data_source_async(&self, req: &super::api::Empty) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::DataSourceResponse>> {
        self.get_data_source_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn send_metrics_opt(&self, req: &super::api::MetricsRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::Empty> {
        self.client.unary_call(&METHOD_INSPEKTOR_SEND_METRICS, req, opt)
    }

    pub fn send_metrics(&self, req: &super::api::MetricsRequest) -> ::grpcio::Result<super::api::Empty> {
        self.send_metrics_opt(req, ::grpcio::CallOption::default())
    }

    pub fn send_metrics_async_opt(&self, req: &super::api::MetricsRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::Empty>> {
        self.client.unary_call_async(&METHOD_INSPEKTOR_SEND_METRICS, req, opt)
    }

    pub fn send_metrics_async(&self, req: &super::api::MetricsRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::Empty>> {
        self.send_metrics_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_integration_config_opt(&self, req: &super::api::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::api::IntegrationConfigResponse> {
        self.client.unary_call(&METHOD_INSPEKTOR_GET_INTEGRATION_CONFIG, req, opt)
    }

    pub fn get_integration_config(&self, req: &super::api::Empty) -> ::grpcio::Result<super::api::IntegrationConfigResponse> {
        self.get_integration_config_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_integration_config_async_opt(&self, req: &super::api::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::IntegrationConfigResponse>> {
        self.client.unary_call_async(&METHOD_INSPEKTOR_GET_INTEGRATION_CONFIG, req, opt)
    }

    pub fn get_integration_config_async(&self, req: &super::api::Empty) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::api::IntegrationConfigResponse>> {
        self.get_integration_config_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Inspektor {
    fn auth(&mut self, ctx: ::grpcio::RpcContext, req: super::api::AuthRequest, sink: ::grpcio::UnarySink<super::api::AuthResponse>);
    fn policy(&mut self, ctx: ::grpcio::RpcContext, req: super::api::Empty, sink: ::grpcio::ServerStreamingSink<super::api::InspektorPolicy>);
    fn get_data_source(&mut self, ctx: ::grpcio::RpcContext, req: super::api::Empty, sink: ::grpcio::UnarySink<super::api::DataSourceResponse>);
    fn send_metrics(&mut self, ctx: ::grpcio::RpcContext, req: super::api::MetricsRequest, sink: ::grpcio::UnarySink<super::api::Empty>);
    fn get_integration_config(&mut self, ctx: ::grpcio::RpcContext, req: super::api::Empty, sink: ::grpcio::UnarySink<super::api::IntegrationConfigResponse>);
}

pub fn create_inspektor<S: Inspektor + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_INSPEKTOR_AUTH, move |ctx, req, resp| {
        instance.auth(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_server_streaming_handler(&METHOD_INSPEKTOR_POLICY, move |ctx, req, resp| {
        instance.policy(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_INSPEKTOR_GET_DATA_SOURCE, move |ctx, req, resp| {
        instance.get_data_source(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_INSPEKTOR_SEND_METRICS, move |ctx, req, resp| {
        instance.send_metrics(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_INSPEKTOR_GET_INTEGRATION_CONFIG, move |ctx, req, resp| {
        instance.get_integration_config(ctx, req, resp)
    });
    builder.build()
}
