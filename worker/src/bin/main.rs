use clap::Parser;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{ErrorKind, Read, Write},
    net::{Shutdown, TcpStream},
    process,
    rc::Rc,
    thread,
};

use dotenv::dotenv;

extern crate worker;

use shared::{logger, loop_sleep, network, structs::prelude::*};

use worker::{
    connect::connect_to_server,
    local_env::{self, *},
};
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t= HOST.to_string())]
    server_address: String,

    #[arg(long, default_value_t = *PORT)]
    server_port: u16,

    #[arg(long)]
    worker_name: Option<String>,

    #[arg(long, default_value_t = RUST_ENV.to_string())]
    rust_env: String,
}

fn main() {
    dotenv().ok();

    local_env::check_vars();
    let args = Args::parse();
    let worker_name = args
        .worker_name
        .unwrap_or_else(|| shared::utils::random_string(10));

    logger::setup_logger(&args.rust_env.as_str());

    info!("Worker {} ok", worker_name);

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
        info!("Connecting to server...");

        let addr = match network::get_socket_addr(&args.server_address.as_str(), args.server_port) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to parse address: {}", e);
                continue;
            }
        };

        info!(
            "Connecting to server: {} port ::{}",
            &args.server_address, args.server_port
        );
        let main_stream = match connect_to_server(&args.server_address, args.server_port) {
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
            let stream = match connect_to_server(&args.server_address, args.server_port) {
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
