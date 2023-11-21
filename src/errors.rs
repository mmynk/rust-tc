use thiserror::Error;

#[derive(Debug, Error)]
pub enum TcError {
    #[error("Failed to parse: {0}")]
    Parse(String),
}
