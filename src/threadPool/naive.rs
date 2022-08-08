use crate::Result;
use std::thread;
use super::ThreadPool;

/// A naive implementation of a thread pool
pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_thread_num: u64) -> Result<NaiveThreadPool> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, f: F)
    where
            F: FnOnce() + Send + 'static,
    {
        thread::spawn(f);
    }
}