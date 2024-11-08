#[cfg(test)]
mod test;
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
