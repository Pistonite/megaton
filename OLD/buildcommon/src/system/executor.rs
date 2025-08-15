use derivative::Derivative;
use threadpool::ThreadPool;

/// Simple thread pool executor
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Executor {
    pool: ThreadPool,
}

impl Executor {
    /// Create a new executor with NUM_CPU - 1 workers
    pub fn new() -> Self {
        let n = match num_cpus::get() {
            0..2 => 1,
            n => n - 1,
        };
        Self {
            pool: ThreadPool::new(n),
        }
    }

    /// Execute a task
    pub fn execute<T>(&self, f: impl FnOnce() -> T + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.pool.execute(move || {
            let _ = send.send(f());
        });
        Task { recv }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.pool.join();
    }
}

/// A task handled for a spawned task in the executor
pub struct Task<T> {
    recv: oneshot::Receiver<T>,
}

impl<T> Task<T> {
    /// Wait for the task to complete and return the result
    pub fn wait(self) -> T {
        self.recv.recv().unwrap()
    }
}
