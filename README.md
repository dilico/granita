<div align="center">
  <img src="assets/granita.png" alt="Granita" width="200"/>
</div>

# Granita

A load testing framework for simulating realistic traffic and measuring system performance.

## Overview

Granita is a Rust library that provides a simple and flexible API for defining and executing load test scenarios. It allows you to simulate HTTP requests and measure how your system performs under load.

## Usage

Add Granita to your `Cargo.toml`:

```toml
[dependencies]
granita = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Example

Here's a simple example that demonstrates how to use Granita:

```rust
use granita::{Granita, Request, scenario_fn};
use granita::request::HttpRequest;

#[tokio::main]
async fn main() -> Result<(), granita::Error> {
    Granita::new()
        .scenario("fetch_homepage", scenario_fn!(|ctx| {
            let request = HttpRequest::get("https://example.com")
                .build()
                .map_err(|_| granita::Error::Configuration("Invalid URL".into()))?;
            
            let response = ctx.send(Request::Http(request)).await?;
            
            // Process response...
            match response {
                granita::Response::Http(http_response) => {
                    println!("Status: {}", http_response.status);
                }
            }
            
            Ok(())
        }))
        .run()
        .await
}
```

## Features

- Simple builder API for defining test scenarios
- HTTP request/response handling
- Context management for sharing state between requests
- Async/await support

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).
