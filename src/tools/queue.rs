use {
    super::Message,
    crate::Lock,
    std::collections::{vec_deque::IterMut, VecDeque},
};

#[derive(Clone)]
pub struct Queue {
    data: VecDeque<Message>,
    capacity: Lock<usize>,
}

impl Queue {
    pub fn new(capacity: Lock<usize>) -> Self {
        Self {
            data: VecDeque::new(),
            capacity,
        }
    }

    pub fn enqueue<T: AsRef<str>>(&mut self, s: T, owner: String, drafted: bool) {
        let n = self.data.len();

        self.data.push_front(Message::new(s, owner, drafted));

        if let Ok(capacity) = self.capacity.read() {
            if n == *capacity {
                drop(capacity);
                self.data.pop_back();
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<Message> {
        self.data.iter_mut()
    }
}
