use std::sync::Arc;

use crate::engine::http_client::HyperClient;
use crate::engine::request_executor::RequestExecutor;
use crate::error::Error;
use crate::request::{Request, Response};

/// A context for a test run.
pub struct Context {
    executor: RequestExecutor,
}

impl Context {
    /// Creates a new test context.
    pub fn new() -> Self {
        let http_client = HyperClient::new();
        Self { executor: RequestExecutor::new(Arc::new(http_client)) }
    }
    /// Sends a request to the request executor.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to send.
    ///
    /// # Returns
    ///
    /// * `Ok(response)` - The response from the request.
    /// * `Err(error)` - The error that occurred.
    pub async fn send(&self, request: Request) -> Result<Response, Error> {
        let response = self
            .executor
            .execute(request)
            .await
            .map_err(|_| Error::FailedRequestExecution)?;
        Ok(response)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
