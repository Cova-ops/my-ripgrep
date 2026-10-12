use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread::JoinHandle,
};

#[derive(Debug)]
pub struct ThreadPool<T> {
    result_thread: JoinHandle<Vec<T>>,
    threads: Vec<JoinHandle<()>>,
    tx: Option<Sender<Box<dyn FnOnce() -> T + Send + 'static>>>,
    tx_result: Option<Sender<T>>,
}

impl<T> ThreadPool<T>
where
    T: Send + 'static,
{
    pub fn new(max_threads: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() -> T + Send + 'static>>();
        let (tx_result, rx_result) = mpsc::channel::<T>();

        let rx = Arc::new(Mutex::new(rx));

        let result_thread = std::thread::spawn(move || {
            let mut out = vec![];
            loop {
                let msg = rx_result.recv();

                match msg {
                    Ok(v) => out.push(v),
                    Err(_) => break,
                }
            }

            out
        });

        let mut threads = Vec::with_capacity(max_threads);
        for _ in 0..max_threads {
            let rx_worker = Arc::clone(&rx);
            let tx_worker = tx_result.clone();

            threads.push(std::thread::spawn(move || {
                loop {
                    let msg = {
                        let lock = rx_worker.lock().unwrap();
                        lock.recv()
                    };

                    let result = match msg {
                        Ok(func) => func(),
                        Err(_) => break,
                    };

                    tx_worker.send(result).unwrap();
                }
            }));
        }

        Self {
            result_thread,
            threads,
            tx: Some(tx),
            tx_result: Some(tx_result),
        }
    }

    pub fn execute<F>(&self, func: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.tx.as_ref().unwrap().send(Box::new(func)).unwrap();
    }

    pub fn join(mut self) -> Vec<T> {
        drop(self.tx.take());

        for t in self.threads {
            t.join().unwrap();
        }

        drop(self.tx_result.take());
        self.result_thread.join().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executes_all_jobs_and_collects_results() {
        let pool = ThreadPool::new(3);
        for i in 0..20 {
            pool.execute(move || i * i);
        }

        let mut results = pool.join();
        results.sort_unstable();

        let mut expected: Vec<_> = (0..20).map(|i| i * i).collect();
        expected.sort_unstable();

        assert_eq!(results, expected);
    }
    #[test]
    fn joining_empty_pool_returns_empty_vec() {
        let pool: ThreadPool<usize> = ThreadPool::new(2);
        assert_eq!(pool.join(), []);
    }
}
