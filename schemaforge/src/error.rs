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

pub fn format_for_tests(err: &Error) -> String {
    let mut rendered = err.to_string();
    let mut source = std::error::Error::source(err);
    while let Some(cause) = source {
        rendered.push_str("\ncaused by: ");
        rendered.push_str(&cause.to_string());
        source = cause.source();
    }
    rendered
}
