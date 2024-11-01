mod workers;

use crate::workers::Workers;
use criterion::black_box;
use futures::stream::{FuturesUnordered, StreamExt};
use rayon::scope;

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn do_something(value: usize) {
    black_box(value);
    assert_eq!(fibonacci(8), 21);
}

pub async fn future_unordered(count: usize) {
    let mut tasks = FuturesUnordered::new();
    for i in 0..count {
        let task = async move { do_something(i) };
        tasks.push(task);
    }
    while let Some(()) = tasks.next().await {}
}

pub async fn future_unordered_pooled(max_concurrent: usize, mut count: usize) {
    let mut tasks = FuturesUnordered::new();
    while count > 0 || !tasks.is_empty() {
        // Fill up to max_concurrent tasks
        while tasks.len() < max_concurrent && count > 0 {
            let task = async move { do_something(count) };
            tasks.push(task);
            count -= 1;
        }
        // Poll the tasks and await for the next completed one
        if let Some(()) = tasks.next().await {}
    }
    while let Some(()) = tasks.next().await {}
}

pub fn rayon(count: usize) {
    scope(|s| {
        for i in 0..count {
            s.spawn(move |_| {
                do_something(i);
            });
        }
    });
}

pub async fn tokio_tasks(max_concurrent: usize, count: usize) {
    let mut workers = Workers::with_capacity(max_concurrent, do_something);
    for i in 0..count {
        workers.send(i).await;
    }
    workers.join().await;
}

#[cfg(test)]
mod test {
    use crate::{future_unordered, future_unordered_pooled, rayon, tokio_tasks};

    const COUNT: usize = 100_000_000;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered() {
        future_unordered(COUNT).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered_pooled() {
        future_unordered_pooled(num_cpus::get(), COUNT).await;
    }

    #[test]
    fn test_rayon() {
        rayon(COUNT);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tokio_tasks() {
        tokio_tasks(num_cpus::get(), COUNT).await;
    }
}
