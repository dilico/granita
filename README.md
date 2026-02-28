<div align="center">
  <img src="assets/granita.png" alt="Granita" width="200"/>
</div>

# Granita

A load testing framework for simulating realistic traffic and measuring system performance.

[![License][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[license-badge]: https://img.shields.io/github/license/dilico/granita
[license-url]: https://github.com/dilico/granita/blob/main/LICENSE

[actions-badge]: https://github.com/dilico/granita/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/dilico/granita/actions/workflows/ci.yml

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

Define a scenario as a sequence of requests. Add one or more scenarios to a load test, then run.

```rust
use granita::{Granita, Scenario};
use granita::request::HttpRequest;

#[tokio::main]
async fn main() -> Result<(), granita::Error> {
    let fetch_homepage = Scenario::new("fetch_homepage")
        .request(HttpRequest::get("https://example.com")
        .build()
        .unwrap());

    Granita::new()
        .scenario(fetch_homepage)
        .run()
        .await
}
```

For response-dependent steps (e.g. use data from a previous response in the next request), implement the `Step` trait and add steps with `.step(your_step)`.

## Features

- Simple API: add scenarios and requests
- Scenarios as ordered sequences of requests and steps
- Static requests or dynamic steps (implement `Step` for response-dependent requests)
- HTTP request/response handling and async/await support

## License

Licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).
