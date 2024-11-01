# sdb-pipeline

```bash
cargo test --release -- --test-threads=1
```

```bash
cargo bench
```

```bash
parallelism/future_unordered
                        time:   [507.11 ms 509.81 ms 512.25 ms]
parallelism/future_unordered_pooled
                        time:   [396.08 ms 396.84 ms 397.63 ms]
parallelism/tokio_tasks
                        time:   [218.91 ms 232.03 ms 246.85 ms]
parallelism/rayon
                        time:   [1.5305 s 1.5997 s 1.6741 s]
```