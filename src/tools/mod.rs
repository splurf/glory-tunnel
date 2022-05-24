mod handler;
mod message;
mod queue;
mod tunnel;

use {
    handler::create_handler,
    message::Message,
    queue::Queue,
    std::sync::{Arc, RwLock},
};

pub use tunnel::Tunnel;

// Using RwLock instead Mutex because Readable and Writable access is allowed
type Lock<T> = Arc<RwLock<T>>;

fn lock<T>(element: T) -> Lock<T> {
    Arc::new(RwLock::new(element))
}
