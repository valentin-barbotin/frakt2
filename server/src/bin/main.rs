use clap::Parser;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::Write,
};

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
struct Args {
    #[arg(long, default_value_t= HOST.to_string())]
    host: String,

    #[arg(long, default_value_t= *PORT)]
    port: u16,

    #[arg(long, default_value_t= RUST_ENV.to_string())]
    rust_env: String,

}

fn main() {
    dotenv().ok();

    local_env::check_vars();

    let args = Args::parse();

    logger::setup_logger(&args.rust_env.as_str());

    info!("Starting server on port {}", *PORT);

    listener::start_server(&args.host, args.port);
}
