use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    process, thread,
};

use dotenv::dotenv;

extern crate worker;

use shared::{
    colors,
    network,
    structs::prelude::*
};

use worker::{
    connect::connect_to_server,
    local_env::{self, *},
};

macro_rules! loop_sleep {
    ($duration:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($duration));
    };
}
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

    let worker_name = "[worker name]";

    info!("Worker {} ok", worker_name);

    let duration = match RUST_ENV.as_str() {
        "debug" => 500,
        "trace" => 500,
        _ => 10,
    };
    
    loop {
        loop_sleep!(duration);

        let main_stream = match connect_to_server() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to server: {}", e);
                continue;
            }
        };

        let fragment = Fragment::Request(FragmentRequest::new(worker_name, 500));

        match network::send_message(&main_stream, fragment, None, None) {
            Ok(_) => info!("Fragment request sent"),
            Err(e) => error!("Failed to send message: {}", e),
        }

        let task = match get_task(&main_stream) {
            Some(t) => t,
            None => {
                network::close_stream(main_stream);
                continue;
            }
        };

        let stream = match connect_to_server() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to server: {}", e);
                continue;
            }
        };

        let mut task = Box::new(task);

        loop {
            loop_sleep!(duration);

            network::handle_response(&stream, task.0, task.1);

            *task = match get_task(&stream) {
                Some(t) => t,
                None => {
                    network::close_stream(stream);
                    break;
                }
            };
        }
    }
}

fn get_task(stream: &TcpStream) -> Option<(String, Vec<u8>)> {
    let response = match network::receive_message(&stream) {
        Ok(r) => r,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::ConnectionAborted {
                error!("Server connection closed.");
            } else {
                error!("Failed to get message from stream: {}", e);
            }
            
            return None;
        }
    };

    Some(response)
}