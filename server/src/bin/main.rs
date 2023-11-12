use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    thread,
};

use dotenv::dotenv;

extern crate server;

use shared::colors;

use server::{
    listener,
    local_env::{self, *},
};

fn main() {
    dotenv().ok();

    local_env::check_vars();

    // Initialize logger
    let level: LevelFilter = match RUST_ENV.as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Error => colors::RED,
                log::Level::Warn => colors::YELLOW,
                log::Level::Info => colors::GREEN,
                log::Level::Debug => colors::BLUE,
                log::Level::Trace => colors::CYAN,
            };
            writeln!(
                buf,
                "[{}{}{}] - {}",
                level,
                record.level(),
                colors::RESET,
                record.args()
            )
        })
        .filter(None, level)
        .target(env_logger::Target::Stdout)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    info!("Starting server on port {}", *PORT);

    listener::start_server();
}
