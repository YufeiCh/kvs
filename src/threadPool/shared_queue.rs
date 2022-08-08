use std::thread;

use crossbeam::channel::{self, Sender, Receiver};

use crate::Result;
use super::ThreadPool;

use log::{debug, error};

/// a shared_queue thread pool
/// when thread panic, it will spawn a new thread
pub struct  SharedQueueThreadPool {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u64) -> Result<Self> {
        let (tx, rx) = channel::unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..threads {
            let rx = TaskReciever(rx.clone());
            thread::Builder::new().spawn(move || run_tasks(rx))?;
        }
        Ok(SharedQueueThreadPool {tx})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static {
        self.tx
        .send(Box::new(job))
        .expect("The thread pool has no thread");
    }
}

#[derive(Clone)]
struct TaskReciever(Receiver<Box<dyn FnOnce() + Send + 'static>>);

impl Drop for TaskReciever {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.clone();
            if let Err(e) = thread::Builder::new().spawn(move || run_tasks(rx)) {
                error!("Failed to spawn a thread: {}", e);
            }
        }
    }
}

fn run_tasks(rx: TaskReciever) {
    loop {
        match rx.0.recv() {
            Ok(task) => {
                task();
            }
            Err(_) => debug!("Thread exists because the thread pool is destroyed"),
        }
    }
}