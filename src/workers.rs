use thingbuf::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

pub(super) struct Workers {
    workers: Vec<Worker>,
    handles: Vec<JoinHandle<()>>,
    next: usize,
}

impl Workers {
    pub(super) fn with_capacity<F>(max_concurrent: usize, func: F) -> Self
    where
        F: Fn(usize, u32, u64) + 'static + Send + Sync + Copy,
    {
        let mut workers = Vec::with_capacity(max_concurrent);
        let mut handles = Vec::with_capacity(max_concurrent);
        for _ in 0..max_concurrent {
            let (worker, handle) = Worker::new(func);
            workers.push(worker);
            handles.push(handle);
        }
        Self {
            workers,
            handles,
            next: 0,
        }
    }

    pub(super) async fn send(&mut self, value: usize, n: u32, expect: u64) {
        self.workers[self.next].send(value, n, expect).await;
        self.next = (self.next + 1) % self.workers.len();
    }

    pub(super) async fn join(self) {
        for worker in self.workers {
            drop(worker);
        }
        for h in self.handles {
            h.await.unwrap();
        }
    }
}

pub(super) struct Worker(Sender<(usize, u32, u64)>);

impl Worker {
    fn new<F>(func: F) -> (Self, JoinHandle<()>)
    where
        F: Fn(usize, u32, u64) + 'static + Send + Sync,
    {
        let (tx, rx) = channel(10000);
        let handle = tokio::task::spawn(async move {
            while let Some((value, n, expect)) = rx.recv().await {
                func(value, n, expect);
            }
        });
        (Self(tx), handle)
    }

    async fn send(&self, value: usize, n: u32, expect: u64) {
        self.0.send((value, n, expect)).await.unwrap()
    }
}
