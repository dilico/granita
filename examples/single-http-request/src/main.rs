use granita::{Granita, MetricsSinkType, Scenario, request::HttpRequest};

#[tokio::main]
async fn main() {
    let scenario = Scenario::new("example_scenario").request(
        "example_request",
        HttpRequest::get("https://google.com").build().unwrap(),
    );

    let result = Granita::new()
        .scenario(scenario)
        .sink(MetricsSinkType::Console)
        .run()
        .await;
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
