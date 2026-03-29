use log::error;
use serde::Serialize;

pub trait LogErr<T> {
    fn log_err(self, context: &str) -> Self;
}

impl<T, E: std::fmt::Display> LogErr<T> for Result<T, E> {
    fn log_err(self, context: &str) -> Self {
        if let Err(ref e) = self {
            error!("{context}: {e}")
        }
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NWError {
    #[error("No SNIRF file loaded")]
    NoData,
    #[error("No NIRS entries in file")]
    NoEntries,
    #[error("Block index {0} out of range")]
    BlockOutOfRange(usize),
    #[error("Channel {0} not found")]
    ChannelNotFound(usize),
    #[error("Parse error {0}")]
    Parse,
    #[error("State lock poisoned")]
    LockPoisoned,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Internal(String),
}

impl Serialize for NWError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
