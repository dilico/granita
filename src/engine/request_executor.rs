use std::sync::Arc;

#[cfg(test)]
use mockall::predicate::*;
use thiserror::Error;

use crate::{
    Request, Response,
    engine::http_client::{HttpClient, HttpError},
    request::{BuildHttpResponseError, HttpRequestError, HttpResponseBuilder},
};

pub(crate) struct RequestExecutor {
    http_client: Arc<dyn HttpClient + Send + Sync>,
}

impl RequestExecutor {
    pub(crate) fn new(http_client: Arc<dyn HttpClient + Send + Sync>) -> Self {
        Self { http_client }
    }

    pub(crate) async fn execute(
        &self,
        request: Request,
    ) -> Result<Response, RequestExecutorError> {
        match request {
            Request::Http(http_request) => {
                let response =
                    self.http_client.get(http_request.url.clone()).await?;
                Ok(Response::Http(
                    HttpResponseBuilder::new()
                        .status(200)
                        .body(response)
                        .build()?,
                ))
            }
        }
    }
}
#[derive(Debug, Error)]
pub(crate) enum RequestExecutorError {
    #[error("failed to build HTTP request")]
    HttpRequestBuild(#[from] HttpRequestError),

    #[error("failed to build HTTP response")]
    HttpResponseBuild(#[from] BuildHttpResponseError),

    #[error("failed to execute request")]
    FailedExecution(#[from] HttpError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::HttpRequest;
    use crate::{Request, engine::http_client::MockHttpClient};

    #[tokio::test]
    async fn execute_get_http_request_succeeds() {
        let url = "https://example.com";
        let response_body = "success";
        let mut mock_http_client = MockHttpClient::default();
        mock_http_client.expect_get().with(always()).returning(move |_| {
            Box::pin(async move { Ok(response_body.to_string()) })
        });

        let executor = RequestExecutor::new(Arc::new(mock_http_client));
        let request = Request::Http(HttpRequest::get(url).build().unwrap());
        let response = executor.execute(request).await.unwrap();

        match response {
            Response::Http(http) => {
                assert_eq!(http.status, 200);
                assert_eq!(http.body, response_body);
                assert!(http.headers.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn execute_get_http_request_fails() {
        let url = "https://example.com";
        let mut mock_http_client = MockHttpClient::default();
        mock_http_client.expect_get().with(always()).times(1).returning(
            move |_| {
                Box::pin(async {
                    Err(HttpError::Uri("some error".to_string()))
                })
            },
        );

        let executor = RequestExecutor::new(Arc::new(mock_http_client));
        let request = Request::Http(HttpRequest::get(url).build().unwrap());
        let response = executor.execute(request).await;

        assert!(matches!(
            response,
            Err(RequestExecutorError::FailedExecution(HttpError::Uri(_)))
        ))
    }
}
