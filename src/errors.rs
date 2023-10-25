use std::error;

use bincode::ErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetlinkError {
    #[error("Failed to create socket: {0}")]
    Socket(#[from] Box<dyn error::Error>),

    #[error("Failed to send message: {0}")]
    Send(String),

    #[error("Netlink error: {0}")]
    Netlink(String),

    #[error("Failed to decode netlink message: {0}")]
    NetlinkDecode(String),
}

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("rust-netlink error: {0}")]
    Netlink(#[from] NetlinkError),

    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
}

#[derive(Debug, Error)]
pub enum TcError {
    #[error("rust-netlink error: {0}")]
    Netlink(#[from] NetlinkError),

    #[error("Failed to retrieve links: {0}")]
    Link(#[from] LinkError),

    #[error("Failed to decode field: {0}")]
    Decode(String),

    #[error("Failed to unmarshal struct: {0}")]
    UnmarshalStruct(#[from] Box<ErrorKind>),

    #[error("Failed to unmarshal structs: {0}")]
    UnmarshalStructs(String),

    #[error("Inavalid attribute: {0}")]
    InvalidAttribute(String),

    #[error("Attribute not implemented: {0}")]
    UnimplementedAttribute(String),
}
