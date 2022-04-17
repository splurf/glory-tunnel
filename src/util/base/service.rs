#[derive(Clone, Debug)]
pub enum Service {
    Host,
    Client,
}

impl Service {
    pub fn from_str<T: AsRef<str>>(s: T) -> Option<Self> {
        match s.as_ref() {
            "--host" => Some(Self::Host),
            "--connect" => Some(Self::Client),
            _ => None,
        }
    }
}
