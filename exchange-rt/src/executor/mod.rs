use std::future::Future;

mod tokio;
pub(crate) use tokio::TokioExecutor;

/// An executor of futures.
pub trait Executor {
    /// Place the future into the executor to be run.
    fn execute<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
