use crate::{
    buffer_unordered, do_something, fibonacci, for_each_concurrent, future_unordered,
    future_unordered_pooled, rayon_par_iter, rayon_scope, tokio_tasks,
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
fn test_rayon_scope_is_parallel() {
    let cpus = num_cpus::get();
    let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
    let time = Instant::now();
    rayon_scope(cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR);
    assert!(time.elapsed() < fib_time * 2);
}

#[test]
fn test_rayon_scope() {
    rayon_scope(COUNT, FIBONACCI_N, FIBONACCI_EXPECT);
}

#[test]
fn test_rayon_par_iter_is_parallel() {
    let cpus = num_cpus::get();
    let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
    let time = Instant::now();
    rayon_par_iter(cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR);
    assert!(time.elapsed() < fib_time * 2);
}

#[test]
fn test_rayon_par_iter() {
    rayon_par_iter(COUNT, FIBONACCI_N, FIBONACCI_EXPECT);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tokio_tasks_is_parallel() {
    let cpus = num_cpus::get();
    let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
    let time = Instant::now();
    tokio_tasks(cpus, cpus * 10, 2, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
    assert!(time.elapsed() < fib_time * 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tokio_tasks() {
    let cpus = num_cpus::get();
    tokio_tasks(cpus, cpus * 8, COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_buffer_unordered_is_not_parallel() {
    let cpus = num_cpus::get();
    let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
    let time = Instant::now();
    buffer_unordered(cpus, cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
    assert!(time.elapsed() > fib_time * 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_buffer_unordered() {
    buffer_unordered(num_cpus::get(), COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_for_each_concurrent_is_not_parallel() {
    let cpus = num_cpus::get();
    let fib_time = calibrate_fibonacci(FIBONACCI_N_PAR);
    let time = Instant::now();
    for_each_concurrent(cpus, cpus, FIBONACCI_N_PAR, FIBONACCI_EXPECT_PAR).await;
    assert!(time.elapsed() > fib_time * 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_for_each_concurrent() {
    for_each_concurrent(num_cpus::get(), COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await;
}
