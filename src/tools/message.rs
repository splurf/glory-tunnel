#[derive(Clone)]
pub struct Message {
    content: String,
    owner: String,
    drafted: bool,
}

impl Message {
    pub fn new<T: AsRef<str>>(s: T, owner: String, drafted: bool) -> Self {
        Self {
            content: s.as_ref().to_string(),
            owner,
            drafted,
        }
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn owner(&self) -> String {
        self.owner.clone()
    }

    pub fn undraft(&mut self) -> bool {
        if !self.drafted {
            self.drafted = true;
            true
        } else {
            false
        }
    }
}
