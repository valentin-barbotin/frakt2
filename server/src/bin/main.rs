use clap::{App, Arg, Parser};
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    fs, io::Write
};

use toml::Value;
use dotenv::dotenv;

extern crate server;

use shared::{
    logger
};

use server::{
    listener,
    local_env::{self, *},
};

struct Args {
    host: String,

    port: u16,

    rust_env: String,
}

fn main() {
    dotenv().ok();
    
   local_env::check_vars();    
    let contents = match fs::read_to_string("server/Config.toml") {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Unable to read file: {}", e);
            return; 
        }
    };
    
    let parsed_toml: Value = match contents.parse() {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Unable to parse TOML: {}", e);
            return; 
        }
    };
    let matches = App::new("server")
        .arg(Arg::new("host")
            .long("host")
            .takes_value(true)
            .required(false))
        .arg(Arg::new("port")
            .long("port")
            .takes_value(true)
            .required(false))
        .arg(Arg::new("rust_env")
            .long("rust-env")
            .takes_value(true)
            .required(false))
        .get_matches();

    let args = Args {
        host: matches.value_of("host")
            .map(String::from)
            .unwrap_or_else(|| {
                parsed_toml.get("HOST")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| HOST.to_string())
            }),
        port: matches.value_of("port")
            .map(|s| s.parse::<u16>().unwrap_or_default())
            .unwrap_or_else(|| {
                parsed_toml.get("PORT")
                    .and_then(|v| v.as_str())
                    .map(|s| s.parse::<u16>().unwrap_or_default())
                    .unwrap_or_else(|| *PORT)
            }),
        rust_env: matches.value_of("rust_env")
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                parsed_toml.get("RUST_ENV")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| RUST_ENV.to_string())
            }),
    };


    logger::setup_logger(&args.rust_env.as_str());
    info!("Host: {}", args.host);
    info!("Starting server on port {}", args.port);
    info!("Rust Environment: {}", args.rust_env);

    listener::start_server(&args.host, args.port);
}
