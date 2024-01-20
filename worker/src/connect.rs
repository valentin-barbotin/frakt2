use crate::local_env::*;
use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn connect_to_server(server: &str, port: u16) -> Result<TcpStream, std::io::Error> {
    let socketaddr = format!("{}:{}", server, port);

    trace!("Connecting to server: {}", socketaddr);

    let stream = TcpStream::connect(&socketaddr)?;

    trace!("Connected to server: {}", socketaddr);

    Ok(stream)
}
