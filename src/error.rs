use std::io;

#[derive(Debug)]
pub enum FindiError {
    IoError(io::Error),
    Utf8Error(std::string::FromUtf8Error)
}

impl From<io::Error> for FindiError {
    fn from(error: io::Error) -> Self {
        FindiError::IoError(error)
    }
}

impl From<std::string::FromUtf8Error> for FindiError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        FindiError::Utf8Error(error)
    }
}

pub type Result<T> = std::result::Result<T, FindiError>;
