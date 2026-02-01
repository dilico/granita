//! Granita load testing framework library.
//!
//! This library provides core types and modules for defining and executing
//! load tests, including context management, request construction,
//! and error handling.
//!
//! # Examples
//!
//! ```
//! use granita::{Granita, Request, Response};
//! ```

/// Builder for Granita.
pub mod builder;
/// Context management for load testing.
pub mod context;
/// Error types for the load testing framework.
pub mod error;
/// Prelude for the load testing framework.
pub mod prelude;
/// Request and response types for load testing.
pub mod request;
/// Scenario function for load testing.
pub mod scenario;

mod engine;

// Re-export commonly used types at the crate root

pub use builder::Granita;
pub use error::Error;
pub use request::{Request, Response};
