use crate::Result;

mod naive;
mod rayon;
mod shared_queue;

pub use self::naive::NaiveThreadPool;
pub use self::rayon::RayonThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;

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
