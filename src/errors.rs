use std::error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TcError {
    #[error("Failed to create socket: {0}")]
    Socket(#[from] Box<dyn error::Error>),

    #[error("Failed to send message: {0}")]
    Send(String),

    #[error("Netlink error: {0}")]
    Netlink(String),

    #[error("Failed to decode netlink message: {0}")]
    Decode(String),
}
