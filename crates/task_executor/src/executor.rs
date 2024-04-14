use std::future::Future;
use std::pin::Pin;

/// A thread-safe, heap-allocated future, resolving to either success (`T`) or failure (`E`).
///
/// Note: `Future`s must be Pinned in order to enable async/enable. see
/// https://rust-lang.github.io/async-book/04_pinning/01_chapter.html#pinning
/// or
/// https://tokio.rs/tokio/tutorial/async#futures.
pub type ExecutorFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

/// An abstraction for executing tasks, suitable for both CPU-bound and I/O-bound operations.
pub trait TaskExecutor {
    type SpawnBlockingError;
    type SpawnError;

    /// Offloads a blocking task, _ensuring_ the async event loop remains responsive.
    /// It accepts a closure that executes a blocking operation and returns a result.
    fn spawn_blocking<F, T>(&self, task: F) -> ExecutorFuture<T, Self::SpawnBlockingError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;

    /// Offloads a non-blocking task asynchronously.
    /// It accepts a future representing an asynchronous operation and returns a result.
    fn spawn<F, T>(&self, task: F) -> ExecutorFuture<T, Self::SpawnError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;
}
