use std::future::Future;

use crate::Executor;

#[derive(Clone, Copy)]
pub struct TokioExecutor;

impl Executor for TokioExecutor {
    fn execute<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::task::spawn(future);
    }
}
