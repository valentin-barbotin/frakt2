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

fn main() {
    dotenv().ok();

    local_env::check_vars();

    logger::setup_logger(RUST_ENV.as_str());

    info!("Starting server on port {}", *PORT);

    listener::start_server();
}
