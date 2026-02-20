use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("pass error: {0}")]
    Pass(String),
}

impl From<kdl::KdlError> for Error {
    fn from(err: kdl::KdlError) -> Self {
        Error::Parse(err.to_string())
    }
}
