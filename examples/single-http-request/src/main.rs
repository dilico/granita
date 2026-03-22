use granita::{
    Granita, LoadProfile, MetricsSinkType, Scenario, request::HttpRequest,
};

#[tokio::main]
async fn main() {
    let scenario = Scenario::new("example_scenario")
        .request(
            "example_request",
            HttpRequest::get("http://127.0.0.1:3000/granita").build().unwrap(),
        )
        .load_profile(LoadProfile::ConstantIterations {
            vus: 100,
            iterations: 100,
        });

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
