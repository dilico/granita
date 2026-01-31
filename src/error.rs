use thiserror::Error;

/// Errors that can occur in the Granita load testing framework.
#[derive(Debug, Error)]
pub enum Error {
    // /// Network-related errors (connection failures, timeouts, etc.)
    // #[error("Network error: {0}")]
    // Network(String),

    // /// HTTP protocol errors (invalid status codes, malformed responses, etc.)
    // #[error("HTTP error: {0}")]
    // Http(String),
    /// Configuration or validation errors
    #[error("Configuration error: {0}")]
    Configuration(Box<str>),
    // /// Request parsing or validation errors
    // #[error("Invalid request: {0}")]
    // InvalidRequest(String),

    // /// Response parsing errors
    // #[error("Invalid response: {0}")]
    // InvalidResponse(String),
    /// Request execution error
    #[error("Request execution error")]
    FailedRequestExecution,
}
