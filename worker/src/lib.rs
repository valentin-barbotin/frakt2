use std::time::Duration;

use log::{debug, error, info, trace};
use serde_json;
use shared::{
    models::fragments::{
        fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
        fragment_task::FragmentTask,
    },
    networking::{
        read_binary_data, read_json_message, read_message_length, result::NetworkingResult,
        send_message, send_result, worker::Worker,
    },
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn run_worker(worker: Worker) {
    info!("Starting worker: {}", worker.name);
    let mut retries: usize = 0;
    let max_retries: usize = 1000;
    let base_delay_ms: u64 = 5;

    let handle = tokio::spawn(async move {
        loop {
            match run(&worker, &mut retries).await {
                Ok(_) => {
                    retries = 0;
                    info!("Worker task completed.")
                }
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        error!("Worker killed due to multiple errors encountered in a row");
                        break;
                    }

                    let delay_ms = calculate_logarithmic_delay(retries, base_delay_ms);
                    error!(
                        "Worker encountered an error: {}, retry {}/{}, waiting for {} ms before retrying",
                        e, retries, max_retries, delay_ms
                    );

                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
        }
    });

    if let Err(e) = handle.await {
        error!("Worker task panicked: {:?}", e);
    }
}

async fn run(worker: &Worker, retry_count: &mut usize) -> NetworkingResult<()> {
    let server_addr = format!("{}:{}", worker.address, worker.port);
    debug!("Connecting to server at {}", server_addr);
    let mut stream = connect_to_server(&server_addr).await?;

    loop {
        debug!("Sending fragment request");
        send_fragment_request(&mut stream, worker).await?;

        let (signature, task) = read_fragment_task(&mut stream).await?;

        debug!("Performing task");
        let (result, data) = perform_task(&task)?;

        debug!("Sending fragment result");
        let mut stream = connect_to_server(&server_addr).await?;
        send_fragment_result(&result, &mut stream, &data, &signature).await?;

        _ = stream.shutdown().await?;
        *retry_count = 0;
    }
}

fn perform_task(task: &FragmentTask) -> NetworkingResult<(FragmentResult, Vec<u8>)> {
    debug!("Performing FragmentTask: {:?}", task);
    task.perform().map_err(|e| {
        error!("Failed to perform the FragmentTask: {}", e);
        e.into()
    })
}

async fn send_fragment_result(
    result: &FragmentResult,
    stream: &mut TcpStream,
    data: &[u8],
    signature: &[u8],
) -> NetworkingResult<()> {
    debug!(
        "Preparing to send FragmentResult with signature: {:?}",
        signature
    );
    let serialized_fragment_result = result.to_json()?;
    let fragment_result_json = serde_json::to_string(&serialized_fragment_result)?;
    trace!("Serialized FragmentResult: {}", serialized_fragment_result);
    trace!("Sending data: {:?}", data);

    send_result(stream, &fragment_result_json, data, signature)
        .await
        .map_err(|e| {
            error!("Failed to send FragmentResult: {}", e);
            e.into()
        })
}

async fn read_fragment_task(stream: &mut TcpStream) -> NetworkingResult<(Vec<u8>, FragmentTask)> {
    debug!("Reading FragmentTask from stream");
    let message_length = read_message_length(stream).await?;
    let json_length = read_message_length(stream).await?;
    let json_message = read_json_message(stream, json_length as usize).await?;
    let data_message = read_binary_data(stream, (message_length - json_length) as usize).await?;

    trace!("Received JSON message: {}", json_message);
    let task = FragmentTask::from_json(&json_message)?;

    info!("Deserialized FragmentTask successfully");
    debug!("FragmentTask details: {:?}", task);

    Ok((data_message, task))
}

async fn send_fragment_request(stream: &mut TcpStream, worker: &Worker) -> NetworkingResult<()> {
    let request = FragmentRequest::new(worker.name.clone(), worker.maximal_work_load);
    let serialized_request = request.to_json()?;
    let serialized_fragment_request = serde_json::to_string(&serialized_request)?;
    debug!("Sending FragmentRequest: {}", serialized_fragment_request);

    send_message(stream, serialized_fragment_request.as_bytes(), None)
        .await
        .map_err(|e| {
            error!("Failed to send FragmentRequest: {}", e);
            e.into()
        })
}

async fn connect_to_server(addr: &str) -> NetworkingResult<TcpStream> {
    TcpStream::connect(addr).await.map_err(|e| {
        error!("Failed to connect to server at {}: {}", addr, e);
        e.into()
    })
}

fn calculate_logarithmic_delay(retry_count: usize, base_delay_ms: u64) -> u64 {
    let retry_float = retry_count as f64;
    let delay_multiplier = retry_float.log2().ceil() * 4.0;
    let delay = base_delay_ms * delay_multiplier as u64;
    delay.max(base_delay_ms)
}
