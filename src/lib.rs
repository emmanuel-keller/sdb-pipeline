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

fn do_something(value: usize, n: u32, expect: u64) {
    black_box(value);
    assert_eq!(fibonacci(n), expect);
}

pub async fn future_unordered(count: usize, n: u32, expect: u64) {
    let mut tasks = FuturesUnordered::new();
    for i in 0..count {
        let task = async move { do_something(i, n, expect) };
        tasks.push(task);
    }
    while let Some(()) = tasks.next().await {}
}

pub async fn future_unordered_pooled(max_concurrent: usize, mut count: usize, n: u32, expect: u64) {
    let mut tasks = FuturesUnordered::new();
    while count > 0 || !tasks.is_empty() {
        // Fill up to max_concurrent tasks
        while tasks.len() < max_concurrent && count > 0 {
            let task = async move { do_something(count, n, expect) };
            tasks.push(task);
            count -= 1;
        }
        // Poll the tasks and await for the next completed one
        if let Some(()) = tasks.next().await {}
    }
    while let Some(()) = tasks.next().await {}
}

pub fn rayon(count: usize, n: u32, expect: u64) {
    scope(|s| {
        for i in 0..count {
            s.spawn(move |_| {
                do_something(i, n, expect);
            });
        }
    });
}

pub async fn tokio_tasks(max_concurrent: usize, count: usize, n: u32, expect: u64) {
    let mut workers = Workers::with_capacity(max_concurrent, do_something);
    for i in 0..count {
        workers.send(i, n, expect).await;
    }
    workers.join().await;
}

#[cfg(test)]
mod test {
    use crate::{
        do_something, fibonacci, future_unordered, future_unordered_pooled, rayon, tokio_tasks,
    };
    use std::time::{Duration, Instant};

    const COUNT: usize = 100_000_000;
    const FIBONACCI_N: u32 = 8;
    const FIBONACCI_EXPECT: u64 = 21;
    const FIBONACCI_N_PAR: u32 = 40;
    const FIBONACCI_EXPECT_PAR: u64 = 102334155;

    #[test]
    fn test_do_something() {
        do_something(1, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR);
    }

    fn calibrate_fibonacci(n: u32) -> Duration {
        let time = Instant::now();
        fibonacci(n);
        time.elapsed()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered_is_not_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let time = Instant::now();
        future_unordered(num_cpus::get(), FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
        assert!(time.elapsed() > fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered() {
        future_unordered(COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered_pooled_is_not_parallel() {
        let cpus = num_cpus::get();
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let time = Instant::now();
        future_unordered_pooled(cpus, cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
        assert!(time.elapsed() > fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_future_unordered_pooled() {
        future_unordered_pooled(num_cpus::get(), COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
    }

    #[test]
    fn test_rayon_is_parallel() {
        let cpus = num_cpus::get();
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let time = Instant::now();
        rayon(cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR);
        assert!(time.elapsed() < fib_time * 2);
    }

    #[test]
    fn test_rayon() {
        rayon(COUNT, FIBONACCI_N, FIBONACCI_EXPECT);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tokio_tasks_is_parallel() {
        let cpus = num_cpus::get();
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let time = Instant::now();
        rayon(cpus, FIBONACCI_N, FIBONACCI_EXPECT);
        assert!(time.elapsed() < fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tokio_tasks() {
        tokio_tasks(num_cpus::get(), COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
    }
}
