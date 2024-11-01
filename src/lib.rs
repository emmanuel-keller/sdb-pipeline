use criterion::black_box;
use futures::stream::{FuturesUnordered, StreamExt};
use rayon::scope;
use tokio::task::spawn_blocking;

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

async fn do_something_async(value: usize, n: u32, expect: u64) {
    black_box(value);
    spawn_blocking(move || {
        assert_eq!(fibonacci(n), expect);
    })
    .await
    .unwrap();
}

pub async fn future_unordered(count: usize, n: u32, expect: u64) {
    let mut tasks = FuturesUnordered::new();
    for i in 0..count {
        let task = do_something_async(i, n, expect);
        tasks.push(task);
    }
    while let Some(()) = tasks.next().await {}
}

pub async fn future_unordered_blocking(count: usize, n: u32, expect: u64) {
    let mut tasks = FuturesUnordered::new();
    for i in 0..count {
        let task = async move { do_something_blocking(i, n, expect) };
        tasks.push(task);
    }
    while let Some(()) = tasks.next().await {}
}

pub async fn future_unordered_pooled(max_concurrent: usize, mut count: usize, n: u32, expect: u64) {
    let mut tasks = FuturesUnordered::new();
    while count > 0 || !tasks.is_empty() {
        // Fill up to max_concurrent tasks
        while tasks.len() < max_concurrent && count > 0 {
            let task = do_something_async(count, n, expect);
            tasks.push(task);
            count -= 1;
        }
        // Poll the tasks and await for the next completed one
        if let Some(()) = tasks.next().await {}
    }
    while let Some(()) = tasks.next().await {}
}

pub async fn future_unordered_pooled_blocking(
    max_concurrent: usize,
    mut count: usize,
    n: u32,
    expect: u64,
) {
    let mut tasks = FuturesUnordered::new();
    while count > 0 || !tasks.is_empty() {
        // Fill up to max_concurrent tasks
        while tasks.len() < max_concurrent && count > 0 {
            let task = async move { do_something_blocking(count, n, expect) };
            tasks.push(task);
            count -= 1;
        }
        // Poll the tasks and await for the next completed one
        if let Some(()) = tasks.next().await {}
    }
    while let Some(()) = tasks.next().await {}
}

fn do_something_blocking(value: usize, n: u32, expect: u64) {
    black_box(value);
    assert_eq!(fibonacci(n), expect);
}

pub fn rayon(count: usize, n: u32, expect: u64) {
    scope(|s| {
        for i in 0..count {
            s.spawn(move |_| {
                do_something_blocking(i, n, expect);
            });
        }
    });
}

#[cfg(test)]
mod test {
    use crate::{
        fibonacci, future_unordered, future_unordered_blocking, future_unordered_pooled,
        future_unordered_pooled_blocking, rayon,
    };
    use std::time::{Duration, Instant};

    fn calibrate_fibonacci(n: u32) -> Duration {
        let time = Instant::now();
        fibonacci(n);
        time.elapsed()
    }

    const FIBONACCI_N_PAR: u32 = 40;
    const FIBONACCI_EXPECT_PAR: u64 = 102334155;

    #[tokio::test(flavor = "multi_thread")]
    async fn t01_future_unordered_is_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let num_cpus = num_cpus::get();
        let time = Instant::now();
        future_unordered(num_cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
        assert!(time.elapsed() < fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn t02_future_unordered_blocking_is_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let num_cpus = num_cpus::get();
        let time = Instant::now();
        future_unordered_blocking(num_cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
        assert!(time.elapsed() < fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn t03_future_unordered_pooled_is_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let num_cpus = num_cpus::get();
        let time = Instant::now();
        future_unordered_pooled(num_cpus, num_cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
        assert!(time.elapsed() < fib_time * 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn t04_future_unordered_pooled_blocking_is_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let num_cpus = num_cpus::get();
        let time = Instant::now();
        future_unordered_pooled_blocking(num_cpus, num_cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR)
            .await;
        assert!(time.elapsed() < fib_time * 2);
    }

    #[test]
    fn t05_rayon_is_parallel() {
        let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
        let time = Instant::now();
        rayon(num_cpus::get(), FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR);
        assert!(time.elapsed() < fib_time * 2);
    }

    const FIBONACCI_N_NOOP: u32 = 8;
    const FIBONACCI_EXPECT_NOOP: u64 = 21;

    const NOOP_COUNT: usize = 2_000_000;

    #[tokio::test(flavor = "multi_thread")]
    async fn t10_future_unordered_noop() {
        future_unordered(NOOP_COUNT, FIBONACCI_N_NOOP, FIBONACCI_EXPECT_NOOP).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn t11_future_unordered_pooled_noop() {
        let num_cpus = num_cpus::get();
        future_unordered_pooled(
            num_cpus,
            NOOP_COUNT,
            FIBONACCI_N_NOOP,
            FIBONACCI_EXPECT_NOOP,
        )
        .await;
    }

    #[test]
    fn t12_rayon_noop() {
        rayon(NOOP_COUNT, FIBONACCI_N_NOOP, FIBONACCI_EXPECT_NOOP);
    }
}
