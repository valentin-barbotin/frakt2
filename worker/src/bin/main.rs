use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{
        Read,
        Write,
        ErrorKind
    },
    net::{Shutdown, TcpStream},
    process, thread, rc::Rc,
};
use clap::Parser;

use dotenv::dotenv;

extern crate worker;

use shared::{
    colors,
    network,
    structs::prelude::*,
    loop_sleep
};

use worker::{
    connect::connect_to_server,
    local_env::{self, *},
};
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {

    dotenv().ok();

    local_env::check_vars();
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }
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

    let worker_name = shared::utils::random_string(10);

    info!("Worker {} ok", worker_name);
    
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        info!("Connecting to server...");

        let main_stream = match connect_to_server() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to server: {}", e);
                continue;
            }
        };

        let fragment = Fragment::Request(FragmentRequest::new(&worker_name, 500));

        match network::send_message(&main_stream, fragment, None, None) {
            Ok(_) => info!("Fragment request sent"),
            Err(e) => error!("Failed to send message: {}", e),
        }

        let task = match network::receive_message(&main_stream) {
            Ok(t) => t,
            Err(e) => {
                match e.kind() {
                    ErrorKind::ConnectionAborted => {
                        // Stream closed by peer
                        error!("Connection aborted");
                    },
                    ErrorKind::UnexpectedEof => {
                        // No task given
                        warn!("Failed to receive message: EOF");
                    },
                    _ => {
                        error!("Failed to receive message")
                    }
                };

                network::close_stream(main_stream);
                continue;
            }
        };

        let mut task = Box::new(task);

        loop {
            loop_sleep!();

            let stream = match connect_to_server() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to connect to server: {}", e);
                    continue;
                }
            };

            handle_message(&stream, task.0, task.1);

            *task = match network::receive_message(&stream) {
                Ok(t) => t,
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionAborted => {
                            // Stream closed by peer
                            error!("Connection aborted");
                        },
                        ErrorKind::UnexpectedEof => {
                            // No task given
                            warn!("Failed to receive message: EOF");
                        },
                        _ => {
                            error!("Failed to receive task");
                        }
                    };

                    network::close_stream(stream);
                    break;
                }
            };

            network::close_stream(stream);
        }
    }
}

pub fn handle_message(stream: &TcpStream, response: String, src_data: Vec<u8>) {
    let message = match network::extract_message(&response) {
        Some(message) => {
            info!("Message type: {:?}", message);
            message
        }
        None => {
            warn!("Unknown message: {}", response);
            return;
        }
    };

    match message {
        Fragment::Task(task) => {
            let (result, data) = task.run();
            match network::send_message(stream, Fragment::Result(result), Some(data), Some(src_data)) {
                Ok(_) => trace!("Result sent"),
                Err(e) => error!("Can't send message: {}", e),
            }
        },
        _ => {
            error!("Unknown message type: {}", response);
        }
    }
}
