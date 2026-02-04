use std::pin::Pin;

#[cfg(test)]
use mockall::{automock, predicate::*};
use thiserror::Error;

type HttpResult = Result<String, HttpError>;

#[cfg_attr(test, automock)]
pub(crate) trait HttpClient: Send + Sync {
    // async fn get(&self, url: &str) -> Result<String, HttpError>;
    fn get(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'static>>;
}

pub(crate) struct HyperClient {}

impl HyperClient {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl HttpClient for HyperClient {
    fn get(
        &self,
        _url: &str,
    ) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'static>> {
        Box::pin(async move { Ok("dummy response".to_string()) })
    }
}

#[derive(Debug, Error)]
pub(crate) enum HttpError {
    #[cfg(test)]
    #[error("generic error")] //TODO: change it once integrated with Hyper
    GenericError,
}
