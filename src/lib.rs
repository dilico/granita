//! Granita load testing framework library.
//!
//! This library provides core types and modules for defining and executing
//! load tests, including context management, request construction,
//! and error handling.
//!
//! # Examples
//!
//! ```
//! use granita::{Context, Request, Response};
//! ```

/// Context management for load testing.
pub mod context;
/// Error types for the load testing framework.
pub mod error;
/// Request and response types for load testing.
pub mod request;

mod engine;

// Re-export commonly used types at the crate root
pub use context::Context;
pub use error::Error;
pub use request::{HttpRequest, HttpResponse, Request, Response};
