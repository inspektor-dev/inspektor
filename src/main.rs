#![feature(async_closure)]
// Copyright 2021 Balaji (rbalajis25@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![feature(type_alias_impl_trait)]
mod apiproto;
mod auditlog;
mod bytespool;
mod config;
mod policy_evaluator;
mod postgres_driver;
mod mysql_driver;
mod sql;
use apiproto::apiproto::inspektor_client::InspektorClient;
use apiproto::{apiproto::*, AuthInterceptor, InspektorClientCommon};

use aws_config::imds::client;
use clap::{App, Arg};
use config::Config;
use env_logger;
use futures;
use futures::prelude::*;
use grpcio::CallOption;
use grpcio::{ChannelBuilder, EnvBuilder};
use log::*;
use openssl_probe;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Channel;
use tonic::{
    codegen::InterceptedService, metadata::MetadataValue, service::Interceptor, Request, Status,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    // intialize openssl variables
    if openssl_probe::try_init_ssl_cert_env_vars() {
        info!("openssl env vars are initialized");
    }
    let app = App::new("inspektor")
        .version("0.0.1")
        .author("Balaji <rbalajis25@gmail.com>")
        .about("Inspektor is a protocol-aware proxy to enforce access policies")
        .arg(
            Arg::with_name("config_file")
                .short('c')
                .long("config_file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let config_path = app.value_of("config_file").unwrap();
    let mut config = config::read_config(&std::path::PathBuf::from(config_path)).unwrap();
    config.validate().unwrap();

    let mut client: InspektorClient<InspektorClientCommon> =
        get_controlplane_client(&config).await.unwrap();
    // retrive data source. so that we can use that to give as input to evaluate policies.
    // if we don't get any data then there is something wrong with control plane or provided
    // secret token.
    let source = client
        .get_data_source(Request::new(Empty {}))
        .await
        .expect("check whether given secret token is valid or check control plane").into_inner();
    // retrive all the integration config.
    let integration_config = client
        .get_integration_config(Request::new(Empty {}))
        .await
        .expect("error while retriving integration config");
    // start audit worker.
    let audit_sender = auditlog::start_audit_worker(integration_config.into_inner()).await;
    // look for policy changes.
    let policy_watcher = look_for_policy_update(client.clone()).await;
    let driver = postgres_driver::driver::PostgresDriver {
        postgres_config: config.postgres_config.unwrap(),
        policy_watcher: policy_watcher,
        datasource: source,
        client: client,
        token: config.secret_token.as_ref().unwrap().clone(),
        audit_sender: audit_sender,
        ssl_acceptor: None,
    };
    driver.start().await;
}

/// get_controlplane_client returns the controlplane's grpc client and call option.
async fn get_controlplane_client(
    config: &Config,
) -> Result<InspektorClient<InspektorClientCommon>, anyhow::Error> {
    let contolplane_addr = format!("http://{}", config.controlplane_addr.as_ref().unwrap());
    let channel = Channel::from_static(Box::leak(contolplane_addr.into_boxed_str()))
        .connect()
        .await?;
    let client = InspektorClient::with_interceptor(
        channel,
        AuthInterceptor {
            token: config.secret_token.as_ref().unwrap().clone(),
        },
    );
    Ok(client)
}

/// look_for_policy_update will open a streaming connection with controlplane. If it detect any changes in polices,
/// then it propogate the changes to the all listeners.
async fn look_for_policy_update(
    mut policy_client: InspektorClient<InspektorClientCommon>,
) -> watch::Receiver<Vec<u8>> {
    // prepare for wathcing polices.
    let mut policy_reciver = policy_client
        .policy(Request::new(Empty {}))
        .await
        .unwrap()
        .into_inner();

    // let rt = tokio::runtime::Runtime::new().unwrap();
    let (policy_broadcaster, policy_watcher) = watch::channel(Vec::<u8>::new());
    // wait for policy in a different thread. we can use the same thread for other common telementry
    // data.
    tokio::spawn(async move {
        info!("strated watching for polices");
        'policy_watcher: loop {
            let result = match policy_reciver.try_next().await {
                Ok(result) => result,
                Err(e) => {
                    error!("error while retriving policies {:?}", e);
                    continue 'policy_watcher;
                }
            };
            let policy = match result {
                Some(policy) => policy,
                None => continue,
            };
            if let Err(e) = policy_broadcaster.send(policy.wasm_byte_code) {
                error!(
                    "error while sending policy to policy watchers. err: {:?}",
                    e
                );
            }
        };
        ()
    });
    return policy_watcher;
}
