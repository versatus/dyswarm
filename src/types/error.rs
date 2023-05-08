use std::net::SocketAddr;

use qp2p::{ConnectionError, EndpointError, SendError};
use thiserror::Error;
use tracing::error;

pub type BroadcastError = DyswarmError;

/// List of all possible errors related to Broadcasting .
#[derive(Error, Debug)]
pub enum DyswarmError {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Send error: {0}")]
    Send(#[from] SendError),

    #[error("Failed to send to {0} reason: {1}")]
    SendAddr(SocketAddr, SendError),

    #[error("Endpoint error: {0}")]
    Endpoint(#[from] EndpointError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UDP port already in use")]
    EaddrInUse,

    #[error("No known peers found")]
    NoPeers,

    #[error("{0}")]
    Other(String),
}
