use crate::engine::metrics::{GroupedMetricsSnapshot, MetricsSink};

/// A sink that prints metrics to the console.
pub(crate) struct ConsoleSink;

impl MetricsSink for ConsoleSink {
    async fn report(&self, snapshot: &GroupedMetricsSnapshot) {
        for (scenario, scenario_metrics) in snapshot.scenarios_metrics.iter() {
            for (request, request_metrics) in
                scenario_metrics.requests_metrics.iter()
            {
                println!("Scenario: {}, Requests: {}", scenario, request,);
                println!("Inflight: {}", request_metrics.inflight);
                println!("Total: {}", request_metrics.total);
                println!("Successful: {}", request_metrics.successful);
                println!("Failed: {}", request_metrics.failed);
                println!("Dropped: {}", request_metrics.dropped);
                println!("Min: {:?}", request_metrics.min);
                println!("Max: {:?}", request_metrics.max);
                println!("Mean: {:?}", request_metrics.mean);
                println!("P50: {:?}", request_metrics.p50);
                println!("P90: {:?}", request_metrics.p90);
                println!("P99: {:?}", request_metrics.p99);
            }
        }
    }
}
