use http_body_util::{BodyExt, Empty};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    client::conn::http1::handshake,
};
use hyper_util::rt::TokioIo;
use std::pin::Pin;
use tokio::net::TcpStream;

#[cfg(test)]
use mockall::{automock, predicate::*};
use thiserror::Error;

type HttpResult = Result<String, HttpError>;

#[cfg_attr(test, automock)]
pub(crate) trait HttpClient: Send + Sync {
    // async fn get(&self, url: &str) -> Result<String, HttpError>;
    fn get(
        &self,
        url: String,
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
        url: String,
    ) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'static>> {
        Box::pin(async move {
            let url = url.parse::<hyper::Uri>()?;
            let host = url.host().ok_or("No host")?;
            let port = url.port_u16().unwrap_or(80);

            let stream =
                TcpStream::connect(format!("{}:{}", host, port)).await?;
            let io = TokioIo::new(stream);

            let (mut sender, conn) = handshake(io).await?;

            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection failed: {:?}", err);
                }
            });

            let authority = url.authority().unwrap().clone();
            let req = Request::builder()
                .uri(url)
                .header(hyper::header::HOST, authority.as_str())
                .body(Empty::<Bytes>::new())?;

            let res: Response<Incoming> = sender.send_request(req).await?;

            // This is the magic part:
            // .collect() aggregates the stream
            // .to_bytes() converts the aggregation into a contiguous Bytes buffer
            let bytes = res.collect().await?.to_bytes();

            Ok(String::from_utf8(bytes.to_vec())?)
        })
    }
}

#[derive(Debug, Error)]
pub(crate) enum HttpError {
    #[error("HTTP error: {0}")]
    Transport(#[from] hyper::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URI error: {0}")]
    Uri(String),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("HTTP error: {0}")]
    Hyper(#[from] hyper::http::Error),
}

impl From<hyper::http::uri::InvalidUri> for HttpError {
    fn from(e: hyper::http::uri::InvalidUri) -> Self {
        HttpError::Uri(e.to_string())
    }
}
impl From<&str> for HttpError {
    fn from(s: &str) -> Self {
        HttpError::Uri(s.to_string())
    }
}
