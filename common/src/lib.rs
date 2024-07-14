use std::{io::Write, time::SystemTime};

pub struct Message {
    username: String,
    content: String,
    timestamp: SystemTime,
}

impl Message {
    pub fn new(username: String, content: String) -> Self {
        Self { username, content, timestamp: SystemTime::now() }
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn timestamp(&self) -> &SystemTime {
        &self.timestamp
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.write_all(self.username.as_bytes()).unwrap();
        buf.push(254);
        buf.write_all(self.content.as_bytes()).unwrap();
        buf.push(255);
        buf
    }
}
