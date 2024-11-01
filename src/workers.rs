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
        F: Fn(usize) + 'static + Send + Sync + Copy,
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

    pub(super) async fn send(&mut self, value: usize) {
        self.workers[self.next].send(value).await;
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

pub(super) struct Worker(Sender<usize>);

impl Worker {
    fn new<F>(func: F) -> (Self, JoinHandle<()>)
    where
        F: Fn(usize) + 'static + Send + Sync,
    {
        let (tx, rx) = channel(10000);
        let handle = tokio::task::spawn(async move {
            while let Some(value) = rx.recv().await {
                func(value);
            }
        });
        (Self(tx), handle)
    }

    async fn send(&self, value: usize) {
        self.0.send(value).await.unwrap()
    }
}
