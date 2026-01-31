use std::collections::HashMap;

use thiserror::Error;

/// A request to be sent to the request executor.
pub enum Request {
    /// An HTTP request.
    Http(HttpRequest),
}

/// An HTTP request.
pub struct HttpRequest {
    /// The HTTP method to use.
    pub method: Method,
    /// The URL to send the request to.
    pub url: String,
    /// The headers to send with the request.
    pub headers: HashMap<String, String>,
}

impl HttpRequest {
    /// Creates a new GET request.
    pub fn get(url: impl Into<String>) -> Self {
        HttpRequest {
            method: Method::Get,
            url: url.into(),
            headers: HashMap::new(),
        }
    }
    /// Adds a header to the request.
    pub fn header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    /// Builds the request.
    pub fn build(self) -> Result<Self, HttpRequestError> {
        if self.url.is_empty() {
            return Err(HttpRequestError::InvalidUrl);
        }
        Ok(self)
    }
}

/// The HTTP method to use.
#[derive(Debug)]
pub enum Method {
    /// The GET method.
    Get,
    /// The POST method.
    Post,
}

/// The error that can occur when building a HTTP request.
#[derive(Debug, Error)]
pub enum HttpRequestError {
    /// The URL is invalid.
    #[error("invalid URL")]
    InvalidUrl,
}

/// A response from the request executor.
#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    /// An HTTP response.
    Http(HttpResponse),
}

/// An HTTP response.
#[derive(Debug, PartialEq, Eq)]
pub struct HttpResponse {
    /// The status code of the response.
    pub status: u16,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    pub body: String,
}

/// A builder for HTTP responses.
#[derive(Debug, Default)]
pub struct HttpResponseBuilder {
    status: Option<u16>,
    headers: HashMap<String, String>,
    body: Option<String>,
}

/// The error that can occur when building a HTTP response.
#[derive(Debug, Error)]
pub enum BuildHttpResponseError {
    /// The status is required.
    #[error("status is required")]
    MissingStatus,
}

impl HttpResponseBuilder {
    /// Creates a new HTTP response builder.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the status of the response.
    pub fn status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }
    /// Inserts a header into the response.
    pub fn insert_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    /// Sets the body of the response.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    /// Builds the response.
    pub fn build(self) -> Result<HttpResponse, BuildHttpResponseError> {
        Ok(HttpResponse {
            status: self
                .status
                .ok_or(BuildHttpResponseError::MissingStatus)?,
            headers: self.headers,
            body: self.body.unwrap_or("".to_string()),
        })
    }
}
