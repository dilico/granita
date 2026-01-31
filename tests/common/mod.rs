//! Server module for the Granita load testing framework.
//!
//! This module contains the helper function to start a test HTTP server and return its URL.

// use axum::Router;

// / Starts a test HTTP server and returns its URL.
// /
// / # Arguments
// /
// / * `router` - The router to serve.
// /
// / # Returns
// /
// / * `url` - The URL of the server.
// pub(crate) async fn start_test_server() -> String {
//     let listener =
//         tokio::net::TcpListener::bind("127.0.0.1:6006").await.unwrap();
//     let addr = listener.local_addr().unwrap();
//     let url = format!("http://{}", addr);
//     // tokio::spawn(async move {
//     //     axum::serve(listener, router).await.unwrap();
//     // });
//     url
// }
