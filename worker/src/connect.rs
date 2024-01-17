use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
    net::{TcpStream, SocketAddr},
};
use crate::local_env::*;

/*
    Connect to a provided address
 */
pub fn connect_to_server(addr: SocketAddr) -> Result<TcpStream, std::io::Error> {
    trace!("Connecting to server: {:?}", addr);
    let stream = TcpStream::connect(addr)?;
    trace!("Connected to server: {:?}", addr);

    Ok(stream)
}
