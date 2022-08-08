use crate::Result;
/// Trait for a thread pool
pub trait ThreadPool {
    /// create a new thread pool with a given pool size
    fn new(size: u64) -> Result<Self>
    where
        Self: Sized;
    
    /// spawn a function in the given thread pool
    fn spawn<F>(&self, job: F)
    where
    F: FnOnce() + Send + 'static;   
}

pub use naive::NaiveThreadPool;

mod naive;