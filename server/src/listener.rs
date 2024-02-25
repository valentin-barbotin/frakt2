use log::{debug, error, info, trace, warn, LevelFilter};
use std::{
    fmt::Display,
    mem::size_of,
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    sync::mpsc::{self, Sender},
    thread,
};
use std::{
    io::{Cursor, ErrorKind, Read, Write},
    os::fd::AsFd,
    str::from_utf8,
    sync::{Arc, Mutex},
};

use crate::{
    local_env::*,
    pool::{Pool, Worker},
    rendering::launch_graphics_engine,
    tasks,
};

use shared::{
    dtos::rendering_data::RenderingData,
    loop_sleep, network,
    networking::server::{Server, ServerConfig},
    structs::prelude::*,
};

use super::tasks::*;

pub fn start_server(host: &str, port: u16) {
    // Check if host is valid
    let host = match Ipv4Addr::from_str(host) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Could not parse host address: {}", e);
            std::process::exit(1)
        }
    };

    let socketaddr = SocketAddrV4::new(host, port);

    debug!("Binding to address: {}", socketaddr);

    let listener = match TcpListener::bind(socketaddr) {
        Ok(listener) => listener,
        _ => panic!("Could not bind to address: {0}:{1}", host, port),
    };

    info!("Listening on {}", socketaddr);

    let workers_pool = Pool::new();
    let workers_pool = Arc::new(Mutex::new(workers_pool));

    let (tx, rx) = std::sync::mpsc::channel()
        as (
            std::sync::mpsc::Sender<Task>,
            std::sync::mpsc::Receiver<Task>,
        );
    let (render_tx, render_rx) = mpsc::channel::<RenderingData>();

    let address = "localhost";
    let port = 8787;
    let width = 300;
    let height = 300;
    let tiles = 4;
    let portal = false;

    let server_config = ServerConfig::new(address.to_string(), port, width, height, tiles, portal);

    let server = create_server(&server_config, &render_tx);

    let pool = Arc::clone(&workers_pool);
    let worker_pool_handle = thread::spawn(move || {
        loop {
            loop_sleep!();

            let mut pool = match pool.lock() {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to lock workers pool: {}", e);
                    continue;
                }
            };
            debug!("Workers pool locked");

            let worker = match pool.get_random_worker() {
                Some(w) => w,
                None => {
                    error!("No worker found for to send task");
                    continue;
                }
            };

            if worker.task.is_some() {
                warn!("Worker {} is busy", worker.get_worker_name());
                continue;
            }

            debug!("Waiting for task...");
            let task = match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(r) => r,
                Err(_e) => {
                    // warn!("Failed to receive task: {}", e);
                    continue;
                }
            };

            trace!("WORKER = {:?}", worker);

            info!("Sending task {} to worker: {}", task.id, worker.get_worker_name());
            // dbg!(worker.get_stream());

            let task2 = task.clone();

            match network::send_message(worker.get_stream(), Fragment::Task(task2.fragment), Some(task2.data), None) {
                Ok(_) => {
                    info!("Task sent {} to worker: {}", task.id, worker.get_worker_name());
                    worker.set_task(Some(&task.id));
                },
                Err(e) => {
                    error!("Can't send message: {}", e);
                    match tx.send(task) {
                        Ok(_) => trace!("Task sent back to pipe"),
                        Err(e) => error!("Can't send message back: {}", e),
                    };
                },
            };
        }
    });

    let listener_server = server.clone();
    let listener_handle = thread::spawn(move || loop {
        debug!("Waiting for connection...");
        let (stream, addr) = match listener.accept() {
            Ok((stream, addr)) => (stream, addr),
            Err(e) => {
                error!("Could not accept connection: {}", e);
                continue;
            }
        };

        debug!("Connection established: {}", addr);

        let pool = Arc::clone(&workers_pool);
        let client_render_tx = render_tx.clone();
        let listener_server = listener_server.clone();
        thread::spawn(move || {
            debug!("Accepted new connection.");

            handle_client(stream, pool, listener_server, client_render_tx);
        });
    });

    let graphics_server = server.clone();
    _ = launch_graphics_engine(graphics_server, render_rx);

    _ = listener_handle.join().unwrap();
    _ = worker_pool_handle.join().unwrap();
}

fn create_server(config: &ServerConfig, render_tx: &Sender<RenderingData>) -> Arc<Mutex<Server>> {
    let server = Server::new(config.clone(), render_tx.clone());
    Arc::new(Mutex::new(server))
}

fn handle_client(
    stream: TcpStream,
    workers_pool: Arc<Mutex<Pool>>,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
) {
    trace!("HANDLE CLIENT");

    let message = match network::receive_message(&stream) {
        Ok(r) => r,
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

            // network::close_stream(stream);
            return;
        }
    };

    let stream_clone = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to clone stream: {}", e);
            return;
        }
    };

    debug!("Handle message: {}", message.0);
    handle_message(
        stream_clone,
        message.0,
        message.1,
        &workers_pool,
        server,
        render_tx,
    );
    debug!("Message handled");
}

pub fn handle_message(
    stream: TcpStream,
    response: String,
    src_data: Vec<u8>,
    workers_pool: &Arc<Mutex<Pool>>,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
) {
    let message = match network::extract_message(&response) {
        Some(message) => {
            debug!("Message type: {:?}", message);
            message
        }
        None => {
            warn!("Unknown message: {}", response);
            return;
        }
    };

    match message {
        Fragment::Result(result) => {
            debug!("TaskResult: {:?}", result);

            let offset = result.id.offset;
            let count = result.id.count;

            // TODO: check
            let mut cursor = Cursor::new(src_data);
            cursor.set_position(offset as u64);

            let mut data = vec![0; count as usize];

            match cursor.read_exact(&mut data) {
                // check
                Ok(_) => trace!("Data read"),
                Err(e) => {
                    error!("Failed to read data: {}", e);
                    return;
                }
            }

            trace!("BYTES : {:?}", data);

            let task_id = match from_utf8(&data) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to convert data to string: {}", e);
                    return;
                }
            };

            let mut pool = match workers_pool.lock() {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to lock workers pool: {}", e);
                    return;
                }
            };

            trace!("TASK ID : {}", task_id);

            let worker = pool.get_worker_with_task(task_id);
            let worker_name = match worker {
                Some(w) => {
                    info!(
                        "Worker {} => set empty task and close stream",
                        w.get_worker_name()
                    );
                    w.set_task(None);
                    w.close_stream();
                    w.set_stream(stream);
                    w.get_worker_name()
                }
                None => {
                    error!("No worker found with task id: {}", task_id);
                    "[unknown-worker]"
                }
            };

            // TODO: use "result"
            let pixel_intensities: Vec<PixelIntensity> = data
                .chunks_exact(size_of::<PixelIntensity>())
                .filter_map(|chunk| {
                    let zn_bytes = chunk.get(0..4)?.try_into().ok()?;
                    let count_bytes = chunk.get(4..8)?.try_into().ok()?;
                    Some(PixelIntensity {
                        zn: f32::from_be_bytes(zn_bytes),
                        count: f32::from_be_bytes(count_bytes),
                    })
                })
                .collect();

            //NOTE: we currenlty only care about the count
            let iterations: Vec<f64> = pixel_intensities.iter().map(|pi| pi.count as f64).collect();

            let rendering_data = RenderingData {
                result,
                iterations,
                worker: worker_name.to_string(),
            };

            if let Err(e) = render_tx.send(rendering_data) {
                error!("Failed to send rendering data: {}", e);
            }
        }
        Fragment::Request(request) => {
            match workers_pool.lock() {
                Ok(mut pool) => {
                    pool.remove_worker(&request.worker_name);

                    let worker = Worker::from_fragment_request(&request, stream);
                    // dbg!(worker.get_stream());

                    pool.add_worker(worker);
                }
                Err(e) => {
                    error!("Failed to lock workers pool: {}", e);
                }
            }

            // let the mpsc loop some time
            // TODO: rework
            warn!("Request handle sleep...");
            // thread::sleep(std::time::Duration::from_secs(5));
            warn!("Request handle sleep... done !");
        }
        _ => {
            error!("Unknown message type: {}", response);
        }
    }
}
