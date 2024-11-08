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
