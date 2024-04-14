use crate::{bounded_tokio_executor::BoundedTokioExecutor, executor::TaskExecutor};

// use rstest::rstest;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::time::Duration;

#[tokio::test]
async fn test_spawn_cpu_task_concurrency() {
    let max_concurrent_tasks = 2;
    let executor = BoundedTokioExecutor::new(max_concurrent_tasks);
    // Spawn more tasks than the maximum number of concurrent tasks, to test the executor's ability
    // to limit the number of concurrent tasks.
    let n_tasks = max_concurrent_tasks + 3;

    let n_running_tasks = Arc::new(AtomicUsize::new(0));
    let max_n_concurrent_tasks = Arc::new(AtomicUsize::new(0));
    let tasks = (0..n_tasks).map(|_| {
        let n_running_tasks = n_running_tasks.clone();
        let max_n_running_tasks = max_n_concurrent_tasks.clone();

        // Cloning the executor and spawning a thread through tokio simulate running the tasks
        // on separate services (like gateways).
        let executor = executor.clone();
        tokio::spawn(async move {
            executor
                // Spawn a CPU-bound task that increments the running counter of running tasks and
                // updates the max number of concurrent tasks.
                .spawn_blocking(move || {
                    // The task started, increment the counter of currenting runnign tasks.
                    let n_currently_running_tasks = safe_atomic_increment(&n_running_tasks);
                    // simulate CPU-bound task;
                    std::thread::sleep(Duration::from_millis(100));
                    // Update the maximum number of concurrent tasks
                    safe_atomic_update_max(&max_n_running_tasks, n_currently_running_tasks);
                    safe_atomic_decrement(&n_running_tasks);
                })
                .await
                .unwrap();
        })
    });

    // Attempt to run all tasks concurrently.
    let _ = futures::future::join_all(tasks).await;

    // Check that number of concurrent tasks was bounded.
    assert!(
        safe_atomic_get_value(&max_n_concurrent_tasks) <= max_concurrent_tasks,
        "More than {max_concurrent_tasks} tasks ran concurrently.",
    );
}

#[tokio::test]
async fn test_spawn_cpu_task_error_handling() {
    let max_concurrent_tasks = 1;
    let executor = BoundedTokioExecutor::new(max_concurrent_tasks);

    let n_running_tasks = Arc::new(AtomicUsize::new(0));

    // Simulate a task that panics.
    let n_running_tasks_cloned = n_running_tasks.clone();

    let task_result = executor
        .spawn_blocking(move || {
            safe_atomic_increment(&n_running_tasks_cloned.clone());
            panic!("Simulated task failure");
        })
        .await;

    assert!(
        task_result.is_err(),
        "Expected the task to fail but it succeeded."
    );

    // Ensure the semaphore is still usable and no deadlock occurs
    let result = executor
        .spawn_blocking(move || {
            safe_atomic_increment(&n_running_tasks);
            "derp" // some normal return value
        })
        .await;

    assert_eq!(
        result.unwrap(),
        "derp",
        "The semaphore was not properly released after a panic."
    );
}

#[tokio::test]
async fn test_spawn_regular_task_error() {
    let executor = BoundedTokioExecutor::new(2);

    let future = async {
        panic!();
    };
    let result = executor.spawn(future).await;
    assert!(
        result.is_err(),
        "Expected the task to fail but it succeeded."
    );
}

// Helpers.
//
// Atomic operations with `Ordering::SeqCst` ensure updates are consistent across threads.
// `Ordering::SeqCst` stops the compiler from reordering operations which may lead to incorrect
// results in a multi-threaded environment.

// Increment the counter and return the new value.
fn safe_atomic_increment(counter: &Arc<AtomicUsize>) -> usize {
    counter.fetch_add(1, Ordering::SeqCst) + 1
}

fn safe_atomic_decrement(counter: &Arc<AtomicUsize>) {
    counter.fetch_sub(1, Ordering::SeqCst);
}

// Update the current max with the maximum of its current value and the new value.
fn safe_atomic_update_max(current_max: &Arc<AtomicUsize>, new_value: usize) {
    current_max.fetch_max(new_value, Ordering::SeqCst);
}

fn safe_atomic_get_value(number: &Arc<AtomicUsize>) -> usize {
    number.load(Ordering::SeqCst)
}
