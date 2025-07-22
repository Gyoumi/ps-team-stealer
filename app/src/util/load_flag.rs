use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::Notify;

pub const FALSE: u8 = 0;
pub const LOADING: u8 = 1;
pub const TRUE: u8 = 2;

pub struct AsyncLoadFlag {
    state: Arc<AtomicU8>,
    notify: Arc<Notify>,
}

impl AsyncLoadFlag {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(FALSE)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    pub fn set_state(&self, val: u8) {
        self.state.store(val, Ordering::Release);
        self.notify.notify_waiters();
    }

    pub async fn wait_until_loaded(&self) {
        while self.state.load(Ordering::Acquire) == LOADING {
            self.notify.notified().await;
        }
    }

    pub async fn try_load<F, Fut>(&self, loader: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let prev = self.state.compare_exchange(
            FALSE,
            LOADING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match prev {
            Ok(_) => {
                loader().await;
                self.set_state(TRUE);
            }
            Err(LOADING) => {
                self.wait_until_loaded().await;
            }
            Err(TRUE) => {}
            _ => {}
        }
    }
}
