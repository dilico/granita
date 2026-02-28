use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::Error;
use crate::context::Context;
use crate::engine::metrics::{MetricsCollector, MetricsSender};
use crate::scenario::{Scenario, ScenarioStep};

/// A builder for Granita load tests.
pub struct Granita {
    scenarios: Vec<Scenario>,
}

impl Granita {
    /// Creates a new Granita builder.
    pub fn new() -> Self {
        Self { scenarios: Vec::new() }
    }

    /// Adds a scenario to the builder.
    pub fn scenario(mut self, scenario: Scenario) -> Self {
        self.scenarios.push(scenario);
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
                let r = match step {
                    ScenarioStep::Static(request) => request,
                    ScenarioStep::Dynamic(step) => {
                        step.request(&context, &previous_responses).await?
                    }
                };
                let response = context.send(r).await?;
                previous_responses.push(response);
            }
        }
        drop(metrics_sender);
        drain_ack_receiver.await.unwrap();
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
                HttpRequest::get("https://example.com").build().unwrap(),
            ));
        assert_eq!(granita.scenarios.len(), 1);
        assert_eq!(granita.scenarios[0].name, "scenario");
    }

    #[test]
    fn scenario_adds_multiple_scenarios() {
        let granita = Granita::new()
            .scenario(Scenario::new("scenario_1").request(
                HttpRequest::get("https://example.com").build().unwrap(),
            ))
            .scenario(Scenario::new("scenario_2").request(
                HttpRequest::get("https://example.com").build().unwrap(),
            ));
        assert_eq!(granita.scenarios.len(), 2);
        assert_eq!(granita.scenarios[0].name, "scenario_1");
        assert_eq!(granita.scenarios[1].name, "scenario_2");
    }
}
