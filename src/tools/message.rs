#[derive(Clone)]
pub struct Message {
    content: String,
    owner: String,
    drafted: bool,
}

impl Message {
    pub fn new<T: AsRef<str>>(s: T, owner: &str, drafted: bool) -> Self {
        Self {
            content: s.as_ref().to_string(),
            owner: owner.to_string(),
            drafted,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn owner(&self) -> &str {
        &self.owner
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
