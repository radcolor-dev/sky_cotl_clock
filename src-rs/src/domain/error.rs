use serde::Serialize;
use std::{error::Error, fmt};

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainError {
    pub code: &'static str,
    pub message: String,
}

impl DomainError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for DomainError {}

impl From<DomainError> for String {
    fn from(error: DomainError) -> Self {
        error.to_string()
    }
}
