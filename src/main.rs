#![feature(async_closure)]
mod apiproto;
mod config;
mod postgres_driver;
use apiproto::api::*;
use apiproto::api_grpc::*;
use burrego::opa::wasm::{Evaluator};
use burrego::opa::host_callbacks::DEFAULT_HOST_CALLBACKS;
use clap::{App, Arg};
use env_logger;
use log::*;
use futures;
use futures::prelude::*;
use grpcio::{ChannelBuilder, EnvBuilder};
use tokio::sync::watch;
use std::sync::Arc;
fn main() {
    env_logger::init();
    // let driver = postgres_driver::driver::PostgresDriver {postgres_config: config::PostgresConfig::default()};
    // driver.start();
    // let app = App::new("inspektor")
    //     .version("0.0.1")
    //     .author("Balaji <rbalajis25@gmail.com>")
    //     .about("inspector is used to autheticate your data layer")
    //     .arg(
    //         Arg::with_name("config_file")
    //             .short("c")
    //             .long("config_file")
    //             .required(true)
    //             .takes_value(true),
    //     )
    //     .get_matches();
    // let config_path = app.value_of("config_file").unwrap();
    // println!("{:?}", config_path)
    // create grpc connection with control plane.
    let config = config::Config::default();
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(config.controlplane_addr.as_ref().unwrap());
    let client = InspektorClient::new(ch);
    
    // create meta which is used for header based authentication.
    let mut meta_builder = grpcio::MetadataBuilder::new();
    meta_builder
        .add_str(
            "auth-token",
            config.secret_token.as_ref().unwrap(),
        )
        .unwrap();
    let meta = meta_builder.build();
    let opt = grpcio::CallOption::default().headers(meta);

    // retrive data source. so that we can use that to give as input to evaluate policies.
    // if we don't get any data then there is something wrong with control plane or provided
    // secret token.
    let source = client
        .get_data_source_opt(&Empty::default(), opt.clone())
        .expect("check whether given secret token is valid or check control plane");

    
    // prepare for wathcing polices.
    let mut policy_reciver = client.policy_opt(&Empty::default(), opt).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (policy_broadcaster, policy_watcher) = watch::channel(Vec::<u8>::new());

    // wait for policy in a different thread. we can use the same thread for other common telementry 
    // data. 
    std::thread::spawn(move || {
        info!("strated watching for polices");
        rt.block_on(async {
            loop {
                while let Some(policy) = policy_reciver.try_next().await.unwrap() {
                    if let Err(e) =  policy_broadcaster.send(policy.wasm_byte_code) {
                        error!("error while sending policy to policy watchers. err: {:?}", e);
                    }
                }
            }
        });
    });
}
