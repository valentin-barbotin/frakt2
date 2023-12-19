use log::{debug, error, info, trace, warn, LevelFilter};
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};

use super::structs::prelude::*;

pub fn close_stream(stream: TcpStream) {
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => debug!("Connection closed"),
        Err(e) => error!("Can't close stream: {}", e),
    }
}

pub fn receive_message(mut stream: &TcpStream) -> Result<(String, Vec<u8>), std::io::Error> {
    // let timeout_duration = std::time::Duration::from_secs(5);
    // stream.set_read_timeout(Some(timeout_duration))?;

    let mut message_size_buffer = [0; 4];
    trace!("message_size_buffer");

    debug!("Receiving message");

    stream.read_exact(&mut message_size_buffer)?;

    debug!("Message received");

    let message_size = u32::from_be_bytes(message_size_buffer) as usize;

    let mut json_message_size_buffer = [0; 4];
    trace!("json_message_size_buffer");
    stream.read_exact(&mut json_message_size_buffer)?;
    let json_message_size = u32::from_be_bytes(json_message_size_buffer) as usize;

    let mut json_message_buffer = vec![0; json_message_size];
    trace!("json_message_buffer");
    stream.read_exact(&mut json_message_buffer)?;
    let json_message = String::from_utf8(json_message_buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut data_buffer = vec![0; message_size - json_message_size];
    trace!("data_buffer");
    trace!("data size: {}", message_size - json_message_size);
    stream.read_exact(&mut data_buffer)?;

    debug!("Message size: {}", message_size);
    trace!("Json message size: {}", json_message_size);
    trace!("Json message: {}", json_message);
    trace!("Data : {:?}", data_buffer);
    // debug!("Data : {:?}", f64::from_be_bytes(data_buffer.as_slice().to_owned()));
    // debug!("Json message: {}", json_message);

    Ok((json_message, data_buffer))
}


pub fn send_message(mut stream: &TcpStream, fragment: Fragment, data: Option<Vec<u8>>, src_data: Option<Vec<u8>>) -> Result<(), std::io::Error> {
    let serialized_message = fragment.serialize();

    let json_msg_size = serialized_message.len();
    
    let mut total_message_size = json_msg_size;

    if let Some(data) = &data {
        total_message_size += data.len();
    }

    if let Some(src_data) = &src_data {
        total_message_size += src_data.len();
    }

    trace!("Sending message size: {}", total_message_size);
    stream.write_all(&(total_message_size as u32).to_be_bytes())?;
    trace!("Sending json message size: {}", json_msg_size);

    stream.write_all(&(json_msg_size as u32).to_be_bytes())?;
    debug!("Sending json message: {}", serialized_message);
    stream.write_all(serialized_message.as_bytes())?;

    if let Some(src_data) = src_data {
        stream.write_all(&src_data)?;
        debug!("Sending src_data size {}", src_data.len());
    }

    if let Some(data) = data {
        stream.write_all(&data)?;
        debug!("Sending data size {}", data.len());
    }

    debug!("Sending total size: {}", total_message_size);

    Ok(())
}

pub fn extract_message(response: &str) -> Option<Fragment> {
    let v: Value = match serde_json::from_str(response) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to deserialize JSON: {}", e);
            return None;
        }
    };

    match v {
        Value::Object(map) => {
            let (key, value) = match map.into_iter().next() {
                Some((key, value)) => (key, value),
                None => {
                    error!("Unknown message type");
                    return None;
                }
            };

            match key.as_str() {
                "FragmentTask" => {
                    let fragment: FragmentTask = match serde_json::from_value(value) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Failed to get fragment: {}", e);
                            return None;
                        }
                    };
                    Some(Fragment::Task(fragment))
                },
                "FragmentRequest" => {
                    let fragment: FragmentRequest = match serde_json::from_value(value) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Failed to get fragment: {}", e);
                            return None;
                        }
                    };
                    Some(Fragment::Request(fragment))
                },
                "FragmentResult" => {
                    let fragment: FragmentResult = match serde_json::from_value(value) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Failed to get fragment: {}", e);
                            return None;
                        }
                    };
                    Some(Fragment::Result(fragment))
                },
                _ => {
                    error!("Unknown message type: {}", response);
                    None
                }
            }
        }

        _ => {
            error!("Unknown message type");
            None
        }
    }
}
