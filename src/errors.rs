use bincode::ErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse: {0}")]
    Parse(String),
}
