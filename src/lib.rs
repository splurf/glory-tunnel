mod config;
mod tools;

use std::sync::{Arc, RwLock};

pub use {
    config::{Config, Service},
    tools::Tunnel,
};

// Using RwLock instead Mutex because Readable and Writable access is allowed
pub type Lock<T> = Arc<RwLock<T>>;

pub fn lock<T>(element: T) -> Lock<T> {
    Arc::new(RwLock::new(element))
}
