use thingbuf::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

pub(super) struct TokioWorkers<T> {
    workers: Vec<TokioWorker<T>>,
    handles: Vec<JoinHandle<()>>,
    next: usize,
}

impl<T> TokioWorkers<T>
where
    T: Default + Clone + Sync + Send + 'static,
{
    pub(super) fn with_capacity<F>(max_concurrent: usize, queue_capacity: usize, func: F) -> Self
    where
        F: Fn(T) + Send + Sync + Copy + 'static,
    {
        let mut workers = Vec::with_capacity(max_concurrent);
        let mut handles = Vec::with_capacity(max_concurrent);
        for _ in 0..max_concurrent {
            let (worker, handle) = TokioWorker::with_capacity(queue_capacity, func);
            workers.push(worker);
            handles.push(handle);
        }
        Self {
            workers,
            handles,
            next: 0,
        }
    }

    pub(super) async fn send(&mut self, message: T) {
        self.workers[self.next].send(message).await;
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

struct TokioWorker<T>(Sender<T>);

impl<T> TokioWorker<T>
where
    T: Default + Clone + Sync + Send + 'static,
{
    fn with_capacity<F>(capacity: usize, func: F) -> (Self, JoinHandle<()>)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let (tx, rx) = channel(capacity);
        let handle = tokio::task::spawn(async move {
            while let Some(message) = rx.recv().await {
                func(message);
            }
        });
        (Self(tx), handle)
    }

    async fn send(&self, message: T) {
        self.0.send(message).await.unwrap()
    }
}
