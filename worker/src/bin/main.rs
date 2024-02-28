use clap::{App, Arg};

use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    fs, io::{ErrorKind, Read, Write}, net::TcpStream, thread
};

use dotenv::dotenv;
use toml::Value;

extern crate worker;

use shared::{
    logger,
    loop_sleep,
    network,
    structs::prelude::*
};

use worker::{
    connect::connect_to_server,
    local_env::{self, *},
};
struct Args {
    host: String,
    port: u16,
    name: String,
    rust_env: String,
}

fn main() {
    dotenv().ok();

    local_env::check_vars();
    let contents = match fs::read_to_string("worker/Config.toml") {
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
    let matches = App::new("worker")
        .arg(Arg::new("host")
            .long("host")
            .takes_value(true)
            .required(false))
        .arg(Arg::new("port")
            .long("port")
            .takes_value(true)
            .required(false))
        .arg(Arg::new("name")
            .long("name")
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
        name: matches.value_of("name")
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                parsed_toml.get("NAME")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| shared::utils::random_string(10))
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
    




    logger::setup_logger(args.rust_env.as_str());
    info!("Host: {}", args.host);
    info!("Port: {}", args.port);
    info!("Name: {}", args.name);
    info!("Rust Environment: {}", args.rust_env);
    
    info!("Worker {} ok", args.name);

    loop {
        thread::sleep(std::time::Duration::from_secs(1));

        let addr = match network::get_socket_addr(args.host.as_str(), args.port) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to parse address: {}", e);
                continue;
            }
        };

        let main_stream = match connect_to_server(addr) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to connect to server: {}", e);
                continue;
            }
        };

        let fragment = Fragment::Request(FragmentRequest::new(&args.name, 500));

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
                    }
                    ErrorKind::UnexpectedEof => {
                        // No task given
                        warn!("Failed to receive message: EOF");
                    }
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
            let stream = match connect_to_server(addr) {
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
                        }
                        ErrorKind::UnexpectedEof => {
                            // No task given
                            warn!("Failed to receive message: EOF");
                        }
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

/*
    Handle a message received from the server and send the result back
    'src_data' is the data received from the server (id)
*/
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
        // Compute a task and send the result back
        Fragment::Task(task) => {
            let (result, data) = task.run();
            match network::send_message(
                stream,
                Fragment::Result(result),
                Some(data),
                Some(src_data),
            ) {
                Ok(_) => trace!("Result sent"),
                Err(e) => error!("Can't send message: {}", e),
            }
        }
        _ => {
            error!("Unknown message type: {}", response);
        }
    }
}
