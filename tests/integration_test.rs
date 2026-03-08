//! Integration tests for the Granita load testing framework.
//!
//! These tests use the public API of the library and make HTTP calls
//! to a mock server, ensuring the library works correctly end-to-end.

use granita::context::Context;
use granita::request::{HttpRequest, HttpResponse};
use granita::{Granita, LoadProfile, Request, Response, Scenario};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

mod common;

#[tokio::test]
async fn test_http_get_request() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("some body"))
        .mount(&mock_server)
        .await;

    let base_url = format!("{}/test", mock_server.uri());
    let request = HttpRequest::get(base_url).build().unwrap();
    let response = Context::new().send(Request::Http(request)).await.unwrap();

    match response {
        Response::Http(HttpResponse { status, headers, body }) => {
            assert_eq!(status, 200);
            assert!(!body.is_empty());
            assert!(headers.is_empty());
        }
    }
}

#[tokio::test]
async fn test_constant_iterations_request_count() {
    let mock_server = MockServer::start().await;
    let base_url = format!("{}/test", mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .expect(4) // 2 VUs × 2 iterations × 1 step
        .mount(&mock_server)
        .await;

    let scenario = Scenario::new("constant_iterations_scenario")
        .request("single_step", HttpRequest::get(base_url).build().unwrap())
        .load_profile(LoadProfile::ConstantIterations {
            vus: 2,
            iterations: 2,
        });

    let result = Granita::new().scenario(scenario).run().await;
    result.expect("run should succeed");
}
