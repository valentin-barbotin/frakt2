use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

pub fn connect_to_server(addr: SocketAddr) -> Result<TcpStream, std::io::Error> {
    let socketaddr = addr.to_string();
    info!("Connecting to server: {}", &socketaddr);
    let stream = TcpStream::connect(&socketaddr)?;
    info!("Connected to server: {}", &socketaddr);

    Ok(stream)
}
