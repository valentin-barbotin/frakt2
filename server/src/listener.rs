use log::{debug, error, info, trace, warn, LevelFilter};
use std::io::{Read, Write};
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    thread,
};

use crate::local_env::*;

use shared::{
    loop_sleep,
    network::{self, handle_message},
};

pub fn start_server() {
    // Check if host is valid
    let host = match Ipv4Addr::from_str(&*HOST) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Could not parse host address: {}", e);
            std::process::exit(1)
        }
    };

    let socketaddr = SocketAddrV4::new(host, *PORT);

    debug!("Binding to address: {}", socketaddr);

    let listener = match TcpListener::bind(socketaddr) {
        Ok(listener) => listener,
        _ => panic!("Could not bind to address: {0}:{1}", *HOST, *PORT),
    };

    info!("Listening on {}", socketaddr);

    loop {
        debug!("Waiting for connection...");
        let (stream, addr) = match listener.accept() {
            Ok((stream, addr)) => (stream, addr),
            Err(e) => {
                error!("Could not accept connection: {}", e);
                continue;
            }
        };

        debug!("Connection established: {}", addr);

        thread::spawn(move || {
            handle_client(stream);
        });
    }
}

fn handle_client(stream: TcpStream) {
    loop {
        loop_sleep!();

        let message = match network::receive_message(&stream) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to get message from stream: {}", e);
                if e.kind() == std::io::ErrorKind::ConnectionAborted {
                    error!("Connection closed with client.");
                } else {
                    error!("Failed to get message from stream: {}", e);
                }
                
                network::close_stream(stream);
                return;
            }
        };

        handle_message(&stream, message.0, message.1);
    }
}
