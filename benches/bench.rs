use criterion::{criterion_group, criterion_main, Criterion};
use sdb_pipeline::{future_unordered, future_unordered_pooled, rayon, tokio_tasks};
use tokio::runtime::Builder;

fn benchmark(c: &mut Criterion) {
    let rt = Builder::new_multi_thread().build().unwrap();
    let mut group = c.benchmark_group("parallelism");
    group.sample_size(10);

    const COUNT: usize = 5_000_000;
    const FIBONACCI_N: u32 = 8;
    const FIBONACCI_EXPECT: u64 = 21;

    let cpus = num_cpus::get();

    group.bench_function("future_unordered", |b| {
        b.to_async(&rt)
            .iter(|| async { future_unordered(COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await });
    });

    group.bench_function("future_unordered_pooled", |b| {
        b.to_async(&rt).iter(|| async {
            future_unordered_pooled(cpus, COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await
        });
    });

    group.bench_function("tokio_tasks", |b| {
        b.to_async(&rt)
            .iter(|| async { tokio_tasks(cpus, COUNT, FIBONACCI_N, FIBONACCI_EXPECT).await });
    });

    group.bench_function("rayon", |b| {
        b.iter(|| rayon(COUNT, FIBONACCI_N, FIBONACCI_EXPECT));
    });

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
