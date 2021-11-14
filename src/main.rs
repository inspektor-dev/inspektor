mod postgres_driver;
mod config;
use clap::{App, Arg};
use env_logger;
fn main() {
    env_logger::init();
    let driver = postgres_driver::driver::PostgresDriver {postgres_config: config::PostgresConfig::default()};
    driver.start();
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
}
