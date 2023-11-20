use bincode::ErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
}

#[derive(Debug, Error)]
pub enum TcError {
    #[error("Failed to retrieve links: {0}")]
    Link(#[from] LinkError),

    #[error("Failed to parse: {0}")]
    Parse(String),

    #[error("Failed to unmarshal struct: {0}")]
    UnmarshalStruct(#[from] Box<ErrorKind>),

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),

    #[error("Attribute not implemented: {0}")]
    UnimplementedAttribute(String),
}
