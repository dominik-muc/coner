use std::{
    sync::{
        mpsc::{self, Receiver, SendError, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::error::*;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(dead_code)]
pub struct ThreadPool {
    sender: Sender<Job>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity < 1 {
            return Err(ServerError::Internal);
        }

        let (sender, receiver) = mpsc::channel();

        let recv_handle = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::new();

        for id in 0..capacity {
            workers.push(Worker::new(id, recv_handle.clone()))
        }

        Ok(Self { sender, workers })
    }
    pub fn execute<F>(
        &self,
        job: F,
    ) -> std::result::Result<(), SendError<Box<(dyn FnOnce() + Send + 'static)>>>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(job))
    }
}

#[allow(dead_code)]
pub struct Worker {
    id: usize,
    thread: JoinHandle<Job>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            job();
        });
        Self { id, thread }
    }
}
