mod tools;
mod util;

use std::sync::{Arc, RwLock};

pub use {
    tools::Tunnel,
    util::{Config, Script, Service},
};

pub type Lock<T> = Arc<RwLock<T>>;

pub fn lock<T>(element: T) -> Lock<T> {
    Arc::new(RwLock::new(element))
}
