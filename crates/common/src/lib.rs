use std::io;

use thiserror::Error;

pub mod client;
pub mod commands;
pub mod connection;
pub mod frame;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error occured {0}")]
    Io(#[from] io::Error),
    #[error("connection reset by peer")]
    ConnectionResetByPeer,
    #[error(transparent)]
    FrameError(#[from] frame::FrameError),
    #[error("invalid command parsed with value `{0}`")]
    InvalidCommand(String),
    #[error("invalid response parsed with value `{0}")]
    InvalidResponse(String),
    #[error("error command received with contents `{0}`")]
    CommandError(String),
    #[error("received a command with an unexpected type")]
    BadCommandType,
}

pub type Result<T> = core::result::Result<T, Error>;
