use crate::executor::TaskExecutor;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::{self, spawn_blocking};

#[cfg(test)]
#[path = "bounded_tokio_executor_test.rs"]
pub mod test;

#[derive(Clone)]
pub struct BoundedTokioExecutor {
    blocking_tasks_semaphore: Arc<Semaphore>,
}

impl BoundedTokioExecutor {
    pub fn new(max_concurrent_cpu_tasks: usize) -> Self {
        Self {
            blocking_tasks_semaphore: Arc::new(Semaphore::new(max_concurrent_cpu_tasks)),
        }
    }
}

impl TaskExecutor for BoundedTokioExecutor {
    type SpawnBlockingError = tokio::task::JoinError;
    type SpawnError = tokio::task::JoinError;

    /// Spawns a blocking task, ensuring it acquires a permit from the semaphore to limit
    /// the number of concurrent executions.
    /// Note: tokio manages spawn_blocking tasks on a separate threadpool from the asynchronous
    /// tasks, so, subject to benchmarking, this throttling might be unnecessary.
    ///
    /// # Example
    ///
    /// ```
    /// use starknet_task_executor::{
    ///   bounded_tokio_executor::BoundedTokioExecutor, executor::TaskExecutor
    /// };
    ///
    /// tokio_test::block_on(async {
    ///     let executor = BoundedTokioExecutor::new(1);
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
        let semaphore = self.blocking_tasks_semaphore.clone();

        async move {
            let _permit_revoked_on_drop = semaphore.acquire_owned().await.unwrap();
            spawn_blocking(task).await
            // <-- permit revoked here.
        }
    }

    /// Executes a non-blocking task without acquiring a semaphore permit.
    ///
    /// # Example
    ///
    /// ```
    /// use starknet_task_executor::{
    ///   bounded_tokio_executor::BoundedTokioExecutor, executor::TaskExecutor
    /// };
    ///
    /// tokio_test::block_on(async {
    ///     let executor = BoundedTokioExecutor::new(0);
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
        task::spawn(task)
    }
}
