use std::future::Future;

use tokio::runtime::Handle;

use crate::executor::TaskExecutor;

#[derive(Clone)]
pub struct TokioExecutor {
    // Invariant: The handle must remain private to ensure all tasks are spawned
    // via TokioExecutor, maintaining control and consistency.
    handle: Handle,
}

impl TokioExecutor {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }
}

impl TaskExecutor for TokioExecutor {
    type SpawnBlockingError = tokio::task::JoinError;
    type SpawnError = tokio::task::JoinError;

    /// Spawns a task that may block, on a dedicated thread, preventing disruption of the async
    /// runtime.

    /// # Example
    ///
    /// ```
    /// use starknet_task_executor::executor::TaskExecutor;
    /// use starknet_task_executor::tokio_executor::TokioExecutor;
    ///
    /// tokio_test::block_on(async {
    ///     let executor = TokioExecutor::new(tokio::runtime::Handle::current());
    ///     let task = || {
    ///         // Simulate CPU-bound work (sleep/Duration from std and not tokio!).
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///         "FLOOF"
    ///     };
    ///     let result = executor.spawn_blocking(task).await;
    ///     assert_eq!(result.unwrap(), "FLOOF");
    /// });
    /// ```
    fn spawn_blocking<F, T>(
        &self,
        task: F,
    ) -> impl Future<Output = Result<T, Self::SpawnBlockingError>> + Send
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.handle.spawn_blocking(task)
    }

    /// Executes a async, non-blocking task.
    ///
    /// # Example
    ///
    /// ```
    /// use starknet_task_executor::{
    ///   tokio_executor::TokioExecutor, executor::TaskExecutor
    /// };
    ///
    /// tokio_test::block_on(async {
    ///     let executor = TokioExecutor::new(tokio::runtime::Handle::current());
    ///     let future = async {
    ///         // Simulate IO-bound work (sleep/Duration from tokio!).
    ///         tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///         "HOPALA"
    ///     };
    ///     let result = executor.spawn(future).await;
    ///     assert_eq!(result.unwrap(), "HOPALA");
    /// });
    fn spawn<F, T>(&self, task: F) -> impl Future<Output = Result<T, Self::SpawnError>> + Send
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.handle.spawn(task)
    }
}
