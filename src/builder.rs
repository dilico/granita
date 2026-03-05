use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::context::Context;
use crate::engine::metrics::MetricsSink;
use crate::engine::metrics::{MetricsCollector, MetricsEvent, MetricsSender};
use crate::scenario::{Scenario, ScenarioStepRequest};
use crate::{Error, MetricsSinkType};

/// A builder for Granita load tests.
pub struct Granita {
    scenarios: Vec<Scenario>,
    sinks: Vec<MetricsSinkType>,
}

impl Granita {
    /// Creates a new Granita builder.
    pub fn new() -> Self {
        Self { scenarios: Vec::new(), sinks: Vec::new() }
    }

    /// Adds a scenario to the builder.
    pub fn scenario(mut self, scenario: Scenario) -> Self {
        self.scenarios.push(scenario);
        self
    }

    /// Adds a metrics sink to the builder.
    pub fn sink(mut self, sink: MetricsSinkType) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Runs all scenarios in the builder.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All scenarios succeeded.
    /// * `Err(error)` - An error occurred during scenario execution.
    pub async fn run(self) -> Result<(), Error> {
        let context = Context::new();
        let dropped_requests = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = tokio::sync::mpsc::channel(10_000); //TODO set channel size to a reasonable value
        let metrics_sender =
            MetricsSender::new(sender, dropped_requests.clone()); //TODO use metrics sender to send metrics events
        let mut metrics_collector =
            MetricsCollector::new(receiver, dropped_requests.clone());
        let (drain_ack, drain_ack_receiver) = tokio::sync::oneshot::channel();
        let metrics_collector_handle = metrics_collector.start(drain_ack);
        for scenario in self.scenarios {
            let mut previous_responses = Vec::new();
            for step in scenario.steps {
                let r = match step.request {
                    ScenarioStepRequest::Static(request) => request,
                    ScenarioStepRequest::Dynamic(step) => {
                        step.request(&context, &previous_responses).await?
                    }
                };
                let start_time = Instant::now();
                metrics_sender.send(MetricsEvent::RequestStart {
                    scenario: scenario.name.clone(),
                    request: step.name.clone(),
                });
                let response = context.send(r).await?;
                previous_responses.push(response);
                metrics_sender.send(MetricsEvent::RequestEnd {
                    scenario: scenario.name.clone(),
                    request: step.name.clone(),
                    duration: start_time.elapsed(),
                    succeeded: true,
                });
            }
        }
        drop(metrics_sender);
        drain_ack_receiver.await.unwrap();
        let grouped_metrics_snapshot = metrics_collector.get().await;
        for sink in self.sinks {
            sink.report(&grouped_metrics_snapshot).await;
        }
        metrics_collector.shutdown();
        metrics_collector_handle
            .await
            .map_err(|err| Error::FailedMetricsCollector(err.to_string()))?;
        Ok(())
    }
}

impl Default for Granita {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::request::HttpRequest;

    use super::*;

    #[test]
    fn scenario_adds_scenario() {
        let granita =
            Granita::new().scenario(Scenario::new("scenario").request(
                "request",
                HttpRequest::get("https://example.com").build().unwrap(),
            ));
        assert_eq!(granita.scenarios.len(), 1);
        assert_eq!(granita.scenarios[0].name, "scenario");
        assert_eq!(granita.scenarios[0].steps.len(), 1);
        assert_eq!(granita.scenarios[0].steps[0].name, "request");
    }

    #[test]
    fn scenario_adds_multiple_scenarios() {
        let granita = Granita::new()
            .scenario(Scenario::new("scenario_1").request(
                "request_1",
                HttpRequest::get("https://example.com").build().unwrap(),
            ))
            .scenario(Scenario::new("scenario_2").request(
                "request_2",
                HttpRequest::get("https://example.com").build().unwrap(),
            ));
        assert_eq!(granita.scenarios.len(), 2);
        assert_eq!(granita.scenarios[0].name, "scenario_1");
        assert_eq!(granita.scenarios[1].name, "scenario_2");
        assert_eq!(granita.scenarios[0].steps.len(), 1);
        assert_eq!(granita.scenarios[1].steps.len(), 1);
        assert_eq!(granita.scenarios[0].steps[0].name, "request_1");
        assert_eq!(granita.scenarios[1].steps[0].name, "request_2");
    }
}
