use log::{debug, error, info, trace, warn, LevelFilter};
use rand::Rng;
use std::{io::{Read, Write}, net::TcpStream};

use shared::structs::fragment_request::FragmentRequest;

#[derive(Debug)]
pub struct Pool {
    pub workers: Vec<Worker>,
}

#[derive(Debug)]
pub struct Worker {
    worker_name: String,
    maximal_work_load: u32,
    stream: Box<TcpStream>,
    pub task: Option<String>
}

impl Worker {
    pub fn new(worker_name: &str, maximal_work_load: u32, stream: TcpStream) -> Self {
        Self {
            worker_name: worker_name.to_string(),
            maximal_work_load,
            stream: Box::new(stream),
            task: None
        }
    }

    pub fn from_fragment_request(fragment: &FragmentRequest, stream: TcpStream) -> Self {
        Self::new(&fragment.worker_name, fragment.maximal_work_load, stream)
    }

    pub fn get_worker_name(&self) -> &str {
        &self.worker_name
    }

    pub fn get_maximal_work_load(&self) -> u32 {
        self.maximal_work_load
    }

    pub fn get_stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn close_stream(&mut self) {
        match self.stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => debug!("Connection closed"),
            Err(e) => error!("Can't close stream: {}", e),
        }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        self.stream = Box::new(stream);

        info!("Worker {} stream updated", self.get_worker_name());
    }

    pub fn set_task(&mut self, task: Option<&str>) {
        self.task = task.map(|t| t.to_string());
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool {
    // TODO: Default trait (check stream default value..)
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
        }
    }

    pub fn add_worker(&mut self, worker: Worker) {
        if self.workers.iter().any(|w| w.worker_name == worker.worker_name) {
            warn!("Worker {} already exists", worker.worker_name);
            return;
        }

        info!("Worker {} added", worker.worker_name);
        self.workers.push(worker)
    }

    pub fn remove_worker(&mut self, worker_name: &str) {
        self.workers.retain(|w| w.worker_name != worker_name);
        info!("Worker {} removed", worker_name);
    }

    pub fn get_worker(&mut self, worker_name: &str) -> Option<&mut Worker> {
        self.workers.iter_mut().find(|w| w.worker_name == worker_name)
    }


    pub fn get_worker_with_task(&mut self, task: &str) -> Option<&mut Worker> {
        self.workers.iter_mut().find(|w| {
            w.task.as_ref().map(|t| t == task).unwrap_or(false)
        })
    }

    pub fn get_random_worker(&mut self) -> Option<&mut Worker> {
        if self.workers.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.workers.len());

        self.workers.get_mut(index)
    }
}
