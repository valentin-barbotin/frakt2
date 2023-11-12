use log::{debug, error, info, trace, warn, LevelFilter};
use std::io::{Read, Write};
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    thread,
};

use crate::local_env::*;

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

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    // stream write/read
}
