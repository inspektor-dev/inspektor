#![feature(async_closure)]
mod postgres_driver;
mod config;
mod apiproto;
use clap::{App, Arg};
use std::sync::Arc;
use grpcio::{ChannelBuilder, EnvBuilder};
use apiproto::api::*;
use apiproto::api_grpc::*;
use futures::prelude::*;
use futures;

use env_logger;
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
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:5003");
    let client = InspektorClient::new(ch);
    let mut meta_builder = grpcio::MetadataBuilder::new(); 
    meta_builder.add_str("auth-token", "91f88693cf40257fcc40b33568925760454ae2ca462bc2b718857d3a9d13").unwrap();
    let meta = meta_builder.build();
    let opt = grpcio::CallOption::default().headers(meta);
    let source = client.get_data_source_opt(&Empty::default(), opt.clone()).unwrap();
    println!("source {:?}", source);
    let mut reciver = client.policy_opt(&Empty::default(), opt).unwrap();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on( async{

        loop{
        while let Some(feature) = reciver.try_next().await.unwrap() {
            println!("data {:?}", feature);
        }
    }
    });
}
