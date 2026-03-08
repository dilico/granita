use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::context::Context;
use crate::engine::metrics::MetricsSink;
use crate::engine::metrics::{MetricsCollector, MetricsEvent, MetricsSender};
use crate::scenario::{Scenario, ScenarioStepRequest};
use crate::{Error, LoadProfile, MetricsSinkType};

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
        let dropped_requests = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = tokio::sync::mpsc::channel(10_000); //TODO set channel size to a reasonable value
        let metrics_sender =
            MetricsSender::new(sender, dropped_requests.clone()); //TODO use metrics sender to send metrics events
        let mut metrics_collector =
            MetricsCollector::new(receiver, dropped_requests.clone());
        let (drain_ack, drain_ack_receiver) = tokio::sync::oneshot::channel();
        let metrics_collector_handle = metrics_collector.start(drain_ack);
        let mut scenario_handles = Vec::new();
        for scenario in self.scenarios {
            let metrics_sender = metrics_sender.clone();
            let scenario_handle = tokio::spawn(async move {
                match scenario.load_profile {
                    LoadProfile::RunOnce => {
                        let context = Context::new();
                        run_scenario(&context, &scenario, &metrics_sender)
                            .await?;
                        Ok::<(), Error>(())
                    }
                    LoadProfile::ConstantIterations { vus, iterations } => {
                        let mut handles = Vec::new();
                        let scenario = Arc::new(scenario);
                        for _ in 0..vus {
                            let scenario = Arc::clone(&scenario);
                            let metrics_sender = metrics_sender.clone();
                            let handle = tokio::spawn(async move {
                                let context = Context::new();
                                for _ in 0..iterations {
                                    run_scenario(
                                        &context,
                                        &scenario,
                                        &metrics_sender,
                                    )
                                    .await?;
                                }
                                Ok::<(), Error>(())
                            });
                            handles.push(handle);
                        }
                        for handle in handles {
                            handle.await.map_err(|err| {
                                Error::FailedScenarioIteration(err.to_string())
                            })??;
                        }
                        Ok::<(), Error>(())
                    }
                }
            });
            scenario_handles.push(scenario_handle);
        }
        for handle in scenario_handles {
            handle
                .await
                .map_err(|err| Error::FailedScenario(err.to_string()))??;
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

async fn run_scenario(
    context: &Context,
    scenario: &Scenario,
    metrics_sender: &MetricsSender,
) -> Result<(), Error> {
    let mut previous_responses = Vec::new();
    for step in &scenario.steps {
        let r = match &step.request {
            ScenarioStepRequest::Static(request) => request.clone(),
            ScenarioStepRequest::Dynamic(step) => {
                step.request(context, &previous_responses).await?
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
    Ok(())
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
