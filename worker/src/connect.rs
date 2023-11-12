use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::local_env::*;

pub fn connect_to_server() -> Result<TcpStream, std::io::Error> {
    let socketaddr = format!("{}:{}", *HOST, *PORT);

    trace!("Connecting to server: {}", socketaddr);

    let stream = TcpStream::connect(&socketaddr)?;

    trace!("Connected to server: {}", socketaddr);

    Ok(stream)
}
