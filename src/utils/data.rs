use object_pool::Pool;
use std::sync::{Condvar, Mutex};

#[derive(serde::Deserialize)]
pub struct Query {
    pub id: String,
    pub question: String,
}

pub struct CustomPool<T> {
    pool: Pool<T>,
    condvar: Condvar,
    lock: Mutex<()>,
}

impl<T> CustomPool<T> {
    pub fn new<F>(size: usize, init: F) -> Self
    where
        F: Fn() -> T,
    {
        CustomPool {
            pool: Pool::new(size, init),
            condvar: Condvar::new(),
            lock: Mutex::new(()),
        }
    }

    pub fn pull(&self) -> T {
        let mut guard = self.lock.lock().unwrap();
        while self.pool.is_empty() {
            guard = self.condvar.wait(guard).unwrap();
        }
        let model = self.pool.pull(|| unreachable!());
        model.detach().1
    }

    pub fn push(&self, model: T) {
        self.pool.attach(model);
        self.condvar.notify_one();
    }
}
