use log::error;
use serde::Serialize;
use thiserror::Error;

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

#[derive(Debug, Error, Serialize)]
pub enum NWError {
    #[error("{0}")]
    GenericError(String),
}

pub fn test_error(return_error: bool) -> Result<String, NWError> {
    if return_error {
        Err(NWError::GenericError("test error".to_string()))
    } else {
        Ok("Success".to_string())
    }
}
