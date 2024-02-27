use clap::Parser;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    env, fs, io::{ErrorKind, Read, Write}, net::TcpStream, thread
};
use toml::Value;
use dotenv::dotenv;

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
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    port: Option<String>,

    #[arg(long)]
    name: Option<String>,

    #[arg(long)]
    rust_env: Option<String>,
}


fn main() {
    dotenv().ok();

    local_env::check_vars();
    let args = Args::parse();
    let contents = fs::read_to_string("worker/Config.toml").expect("Unable to read file");

    let parsed_toml: Value = contents.parse().expect("Unable to parse TOML");


    let host = args.host
    .or(parsed_toml.get("host").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("HOST").ok())
    .unwrap_or_else(|| "random".to_string());

    let port = args.port
    .or(parsed_toml.get("port").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("port").ok())
    .unwrap_or_else(|| "random".to_string());

    let name = args.name
    .or(parsed_toml.get("name").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("NAME").ok())
    .unwrap_or_else(|| shared::utils::random_string(10));

    let rust_env = args.rust_env
    .or(parsed_toml.get("rust_env").and_then(|v| v.as_str()).map(String::from))
    .or_else(|| env::var("RUST_ENV").ok())
    .unwrap_or_else(|| "random".to_string());

    println!("Host: {:?}", host);
    println!("Port: {:?}", port);
    println!("Name: {:?}", name);
    println!("rust_env: {:?}", rust_env);
    
  
    logger::setup_logger(rust_env.as_str());

    info!("Worker {} ok", name);

    loop {
        thread::sleep(std::time::Duration::from_secs(1));

        let addr = match network::get_socket_addr(host.as_str(), port.parse::<u16>().unwrap()) {
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

        let fragment = Fragment::Request(FragmentRequest::new(&name, 500));

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
