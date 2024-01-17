use log::{debug, error, info, trace, warn, LevelFilter};
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream}
};

use crate::fragment_from_json_value;

use super::structs::prelude::*;

/*
    Close the provided stream
*/
pub fn close_stream(stream: TcpStream) {
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => debug!("Connection closed"),
        Err(e) => error!("Can't close stream: {}", e),
    }
}

/*
    Receive a message from the provided stream
    Get the total message size and the JSON message size then receive the payload
    It returns a tuple containing the JSON message and the data
*/
pub fn receive_message(mut stream: &TcpStream) -> Result<(String, Vec<u8>), std::io::Error> {
    debug!("Receiving message");
    let (message_size, json_message_size) = receive_message_size(&mut stream)?;
    receive_message_payload(&mut stream, json_message_size, message_size - json_message_size)
}

/*
    Receive the message size from the provided stream
    It returns a tuple containing the total message size and the JSON message size
*/
fn receive_message_size(mut buffer: impl Read) -> Result<(u32, u32), std::io::Error> {
    let mut total_message_size_buffer = [0; 4];
    buffer.read_exact(&mut total_message_size_buffer)?;
    let total_message_size = u32::from_be_bytes(total_message_size_buffer);

    let mut json_message_size_buffer = [0; 4];
    buffer.read_exact(&mut json_message_size_buffer)?;
    let json_message_size = u32::from_be_bytes(json_message_size_buffer);

    Ok((total_message_size, json_message_size))
}

/*
    Receive the message payload from the provided stream
    It returns a tuple containing the JSON message and the data
*/
fn receive_message_payload(mut buffer: impl Read, json_message_size: u32, data_size: u32) -> Result<(String, Vec<u8>), std::io::Error> {
    let mut json_message_buffer = vec![0; json_message_size as usize];
    buffer.read_exact(&mut json_message_buffer)?;
    let json_message = String::from_utf8(json_message_buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut data_buffer = vec![0; data_size as usize];
    buffer.read_exact(&mut data_buffer)?;

    Ok((json_message, data_buffer))
}

/*
    Send a message to the provided stream
*/
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

    if total_message_size > u32::MAX as usize {
        error!("Message too big");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Message too big"));
    }

    send_message_size(&mut stream, total_message_size as u32, json_msg_size as u32)?;
    send_message_payload(&mut stream, &serialized_message, src_data, data)?;

    debug!("Sending total size: {}", total_message_size);

    Ok(())
}

/*
    Send the message size to the provided stream
*/
fn send_message_size(mut buffer: impl Write, total: u32, payload_size: u32) -> Result<(), std::io::Error> {
    trace!("Sending message total size: {}", total);
    buffer.write_all(&total.to_be_bytes())?;

    trace!("Sending json message size: {}", payload_size);
    buffer.write_all(&payload_size.to_be_bytes())?;

    Ok(())
}

/*
    Send the message payload to the provided stream
*/
fn send_message_payload(mut buffer: impl Write, payload: &str, src_data: Option<Vec<u8>>, data: Option<Vec<u8>>) -> Result<(), std::io::Error> {

    debug!("Sending json message: {}", payload);
    buffer.write_all(payload.as_bytes())?;

    if let Some(src_data) = src_data {
        buffer.write_all(&src_data)?;
        debug!("Sending src_data size {}", src_data.len());
    }

    if let Some(data) = data {
        buffer.write_all(&data)?;
        debug!("Sending data size {}", data.len());
    }

   Ok(())
}


/*
    Extract a message from the provided JSON string
*/
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
                    fragment_from_json_value!(FragmentTask, Task, value)
                },
                "FragmentRequest" => {
                    fragment_from_json_value!(FragmentRequest, Request, value)
                },
                "FragmentResult" => {
                    fragment_from_json_value!(FragmentResult, Result, value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(0, 0)]
    #[case(77, 42)]
    #[case(12, 32)]
    #[case(4444, 1000)]
    fn test_send_message_size(#[case] total: u32, #[case] payload: u32) {
        let mut buffer: Vec<u8> = Vec::new();

        let _ = send_message_size(&mut buffer, total, payload);
        let total = total.to_be_bytes();
        let payload = payload.to_be_bytes();

        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0..4], total);
        assert_eq!(buffer[4..8], payload);
    }

    #[rstest]
    #[case("{}", None, None)]
    #[case("{}", Some(vec![0, 0, 0, 42]), None)]
    #[case("{}", None, Some(vec![0, 0, 0, 42]))]
    #[case("{}", Some(vec![0, 0, 0, 42]), Some(vec![0, 0, 0, 42]))]
    fn test_send_message_payload(#[case] payload: &str, #[case] src_data: Option<Vec<u8>>, #[case] data: Option<Vec<u8>>) {
        let mut buffer: Vec<u8> = Vec::new();

        let _ = send_message_payload(&mut buffer, payload, src_data.clone(), data.clone());

        let payload = payload.as_bytes();

        let mut expected_buffer = Vec::new();

        expected_buffer.extend_from_slice(payload);
        if let Some(src_data) = src_data {
            expected_buffer.extend_from_slice(&src_data);
        }

        if let Some(data) = data {
            expected_buffer.extend_from_slice(&data);
        }

        assert_eq!(buffer.len(), expected_buffer.len());
        assert_eq!(buffer, expected_buffer);
    }

    #[rstest]
    #[case(0, 0)]
    #[case(77, 42)]
    #[case(12, 32)]
    #[case(4444, 1000)]
    fn test_receive_message_size(#[case] total: u32, #[case] payload: u32) {
        let mut buffer: Vec<u8> = Vec::new();

        let _ = send_message_size(&mut buffer, total, payload);

        let mut buffer = buffer.as_slice();

        assert_eq!(buffer.len(), 8);

        let result = receive_message_size(&mut buffer).unwrap();

        assert_eq!(result.0, total);
        assert_eq!(result.1, payload);
    }

    #[rstest]
    #[case("{}", None, None)]
    #[case("{}", Some(vec![0, 0, 0, 42]), None)]
    #[case("{}", None, Some(vec![0, 0, 0, 42]))]
    #[case("{}", Some(vec![0, 0, 0, 42]), Some(vec![0, 0, 0, 42]))]
    fn test_receive_message_payload(#[case] payload: &str, #[case] src_data: Option<Vec<u8>>, #[case] data: Option<Vec<u8>>) {
        let mut buffer: Vec<u8> = Vec::new();

        let _ = send_message_payload(&mut buffer, payload, src_data.clone(), data.clone());

        let mut buffer = buffer.as_slice();

        let result = receive_message_payload(&mut buffer, payload.len() as u32, data.clone().unwrap_or_default().len() as u32).unwrap();

        assert_eq!(result.0, payload);
        assert_eq!(result.1, data.unwrap_or_default());
    }

    #[rstest]
    fn test_close_stream() {
        
    }
}
