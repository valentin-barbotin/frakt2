use clap::Parser;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    env,fs, io::Write
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

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    port: Option<String>,

    #[arg(long)]
    rust_env: Option<String>,
}


fn main() {
    dotenv::from_filename(".env.server").ok();

    let args = Args::parse();
    let contents = fs::read_to_string("server/Config.toml").expect("Unable to read file");

    let parsed_toml: Value = contents.parse().expect("Unable to parse TOML");


    let host = args.host
    .or(parsed_toml.get("host").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("HOST").ok())
    .unwrap_or_else(|| "random".to_string());

    let port = args.port
    .or(parsed_toml.get("port").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("port").ok())
    .unwrap_or_else(|| "random".to_string());


    let rust_env = args.rust_env
    .or(parsed_toml.get("rust_env").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("RUST_ENV").ok())
    .unwrap_or_else(|| "random".to_string());

    println!("Host: {:?}", host);
    println!("Port: {:?}", port);
    println!("rust_env: {:?}", rust_env);
    
    logger::setup_logger(&rust_env);

    info!("Starting server on port {}", port);

    listener::start_server(&host, port.parse::<u16>().unwrap());
}
