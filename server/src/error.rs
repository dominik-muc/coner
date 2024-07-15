use std::{fmt::Display, sync::mpsc::SendError};

pub enum ServerError{
    AuthenticationFailed,
    Internal
}

impl From<std::io::Error> for ServerError {
    fn from(_value: std::io::Error) -> Self {
        todo!()
    }
}
impl<T> From<SendError<T>> for ServerError {
    fn from(_value: SendError<T>) -> Self {
        todo!()
    }
}

pub type Result<T> = std::result::Result<T, ServerError>;

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unknown server error")
    }
}
