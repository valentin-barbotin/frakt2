use log::{debug, error, info, trace, warn, LevelFilter};
use std::{io::{Read, Write, Cursor, ErrorKind}, sync::{Mutex, Arc}, str::from_utf8, os::fd::AsFd};
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    str::FromStr,
    thread,
};

use crate::{local_env::*, pool::{Pool, Worker}, tasks};

use shared::{
    loop_sleep,
    structs::prelude::*,
    network,
};

use super::tasks::*;

pub fn start_server() {
    // Check if host is valid
    let host = match Ipv4Addr::from_str(&HOST) {
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

    let workers_pool = Pool::new();
    let workers_pool = Arc::new(Mutex::new(workers_pool));
    
    let (tx, rx) = std::sync::mpsc::channel() as (std::sync::mpsc::Sender<Task>, std::sync::mpsc::Receiver<Task>);

    // (test code)
    // let params = FractalParams {
    //     fractal_type: "mandelbrot".to_string(),
    //     resolution: Resolution {
    //         nx: 1000,
    //         ny: 1000
    //     },
    //     max_iteration: 500
    // };

    // let split = 8;
    // if let Ok(tasks) = create_fractal_tasks(params, split) {
    //     info!("Tasks created: {}", tasks.len());
    //     for task in tasks {
    //         tx.send(task);
    //     }
    // }

    let pool = Arc::clone(&workers_pool);
    thread::spawn(move || {
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
            dbg!(worker.get_stream());

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

        let pool = Arc::clone(&workers_pool);
        thread::spawn(move || {
            handle_client(stream, pool);
        });
    }
}

fn handle_client(stream: TcpStream, workers_pool: Arc<Mutex<Pool>>) {
    trace!("HANDLE CLIENT");

    let message = match network::receive_message(&stream) {
        Ok(r) => r,
        Err(e) => {
            match e.kind() {
                ErrorKind::ConnectionAborted => {
                    // Stream closed by peer
                    error!("Connection aborted");
                },
                ErrorKind::UnexpectedEof => {
                    // No task given
                    warn!("Failed to receive message: EOF");
                },
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
    handle_message(stream_clone, message.0, message.1, &workers_pool);
    debug!("Message handled");
}

pub fn handle_message(stream: TcpStream, response: String, src_data: Vec<u8>, workers_pool: &Arc<Mutex<Pool>>) {
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

            match cursor.read_exact(&mut data) { // check
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
            match worker {
                Some(w) => {
                    info!("Worker {} => set empty task and close stream", w.get_worker_name());
                    w.set_task(None);
                    w.close_stream();
                    w.set_stream(stream);
                },
                None => {
                    error!("No worker found with task id: {}", task_id);
                }
            }

            // TODO: use "result"
        },
        Fragment::Request(request
        ) => {
            match workers_pool.lock() {
                Ok(mut pool) => {
                    pool.remove_worker(&request.worker_name);

                    let worker = Worker::from_fragment_request(&request, stream);
                    dbg!(worker.get_stream());

                    pool.add_worker(worker);
                },
                Err(e) => {
                    error!("Failed to lock workers pool: {}", e);
                }
            }

            // let the mpsc loop some time
            // TODO: rework
            warn!("Request handle sleep...");
            thread::sleep(std::time::Duration::from_secs(5));
            warn!("Request handle sleep... done !");
        },
        _ => {
            error!("Unknown message type: {}", response);
        }
    }
}
