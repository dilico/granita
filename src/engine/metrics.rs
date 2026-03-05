#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::Mutex;

/// Max integer that `f64` can represent exactly (53-bit mantissa).
const MAX_EXACT_F64_U64: f64 = (1u64 << 53) as f64;

/// Rounds an `f64` to `u64` for use as nanoseconds, avoiding precision loss for large values.
/// Values in `[0, 2^53]` are exact; larger values are clamped to `u64::MAX`.
/// Negative/NaN are treated as 0.
fn round_f64_to_u64_nanos(value: f64) -> u64 {
    let r = value.round();
    if !r.is_finite() || r < 0.0 {
        return 0;
    }
    if r >= u64::MAX as f64 {
        return u64::MAX;
    }
    // For r <= 2^53 the cast is exact; above that f64 cannot represent every integer,
    // so clamp to the max exactly representable value to avoid wrong rounding.
    if r <= MAX_EXACT_F64_U64 {
        return r as u64;
    }
    MAX_EXACT_F64_U64 as u64
}

use hdrhistogram::Histogram;
use tokio::sync::mpsc::error::TrySendError;
use tokio_util::sync::CancellationToken;

use crate::engine::sinks::console_sink::ConsoleSink;

/// An event that can be recorded in the metrics system.
#[derive(Debug)]
pub(crate) enum MetricsEvent {
    /// An event that indicates a request has started.
    RequestStart { scenario: String, request: String },
    /// An event that indicates a request has ended.
    RequestEnd {
        scenario: String,
        request: String,
        duration: Duration,
        succeeded: bool,
    },
}

/// A metrics snapshot for a group of scenarios.
struct GroupedMetrics {
    scenarios_metrics: HashMap<String, ScenarioMetrics>,
}

impl GroupedMetrics {
    /// Creates a new grouped metrics.
    ///
    /// # Returns
    ///
    /// * `Self` - The new grouped metrics.
    fn new() -> Self {
        Self { scenarios_metrics: HashMap::new() } //TODO: think about preallocating the hashmap
    }

    /// Gets a request metrics. Allocates only when inserting a new scenario or request.
    fn get(&mut self, scenario: &str, request: &str) -> &mut RequestMetrics {
        self.scenarios_metrics
            .entry(scenario.to_string())
            .or_insert_with(ScenarioMetrics::new)
            .requests_metrics
            .entry(request.to_string())
            .or_insert_with(RequestMetrics::new)
    }
}

/// A metrics snapshot for a scenario.
struct ScenarioMetrics {
    pub(crate) requests_metrics: HashMap<String, RequestMetrics>,
}

impl ScenarioMetrics {
    /// Creates a new scenario metrics.
    fn new() -> Self {
        Self { requests_metrics: HashMap::new() } //TODO: think about preallocating the hashmap
    }
}

/// A metrics snapshot for a request.
struct RequestMetrics {
    inflight: i64,
    total: u64,
    successful: u64,
    failed: u64,
    histogram: Histogram<u64>,
}

impl RequestMetrics {
    /// Creates a new request metrics.
    fn new() -> Self {
        Self {
            inflight: 0,
            total: 0,
            successful: 0,
            failed: 0,
            histogram: Histogram::new_with_bounds(1, 60_000_000_000, 3)
                .expect("valid bounds for latency histogram"),
        }
    }
}

/// A snapshot of the grouped metrics.
pub(crate) struct GroupedMetricsSnapshot {
    pub(crate) scenarios_metrics: HashMap<String, ScenarioMetricsSnapshot>,
}

impl GroupedMetricsSnapshot {
    /// Creates a new grouped metrics snapshot.
    fn new() -> Self {
        Self { scenarios_metrics: HashMap::new() } //TODO: think about preallocating the hashmap
    }

    /// Gets a request metrics snapshot. Allocates only when inserting a new scenario or request.
    fn get(
        &mut self,
        scenario: &str,
        request: &str,
    ) -> &mut RequestMetricsSnapshot {
        self.scenarios_metrics
            .entry(scenario.to_string())
            .or_insert_with(ScenarioMetricsSnapshot::new)
            .requests_metrics
            .entry(request.to_string())
            .or_insert_with(RequestMetricsSnapshot::new)
    }
}

/// A snapshot of the scenario metrics.
pub(crate) struct ScenarioMetricsSnapshot {
    pub(crate) requests_metrics: HashMap<String, RequestMetricsSnapshot>,
}

impl ScenarioMetricsSnapshot {
    /// Creates a new scenario metrics snapshot.
    fn new() -> Self {
        Self { requests_metrics: HashMap::new() } //TODO: think about preallocating the hashmap
    }
}

/// A snapshot of the request metrics.
pub(crate) struct RequestMetricsSnapshot {
    pub(crate) inflight: i64,
    pub(crate) total: u64,
    pub(crate) successful: u64,
    pub(crate) failed: u64,
    pub(crate) dropped: u64,
    pub(crate) min: Option<Duration>,
    pub(crate) max: Option<Duration>,
    pub(crate) mean: Option<Duration>,
    pub(crate) p50: Option<Duration>,
    pub(crate) p90: Option<Duration>,
    pub(crate) p99: Option<Duration>,
}

impl RequestMetricsSnapshot {
    /// Creates a new request metrics snapshot.
    fn new() -> Self {
        Self {
            inflight: 0,
            total: 0,
            successful: 0,
            failed: 0,
            dropped: 0,
            min: None,
            max: None,
            mean: None,
            p50: None,
            p90: None,
            p99: None,
        }
    }
}

/// A sender for metrics events.
pub(crate) struct MetricsSender {
    sender: tokio::sync::mpsc::Sender<MetricsEvent>,
    dropped_requests: Arc<Mutex<HashMap<(String, String), u64>>>,
}

impl MetricsSender {
    /// Creates a new metrics sender.
    pub(crate) fn new(
        sender: tokio::sync::mpsc::Sender<MetricsEvent>,
        dropped_requests: Arc<Mutex<HashMap<(String, String), u64>>>,
    ) -> Self {
        Self { sender, dropped_requests }
    }

    /// Sends a metrics event.
    pub(crate) fn send(&self, event: MetricsEvent) {
        if let Err(TrySendError::Full(event) | TrySendError::Closed(event)) =
            self.sender.try_send(event)
        {
            eprintln!("Failed to send metrics event");

            let (scenario, request) = match event {
                MetricsEvent::RequestStart { scenario, request }
                | MetricsEvent::RequestEnd { scenario, request, .. } => {
                    (scenario, request)
                }
            };

            let mut map = self.dropped_requests.try_lock().unwrap();
            *map.entry((scenario, request)).or_default() += 1;
        }
    }
}

/// A collector for metrics events.
pub(crate) struct MetricsCollector {
    receiver: Option<tokio::sync::mpsc::Receiver<MetricsEvent>>,
    metrics: Arc<Mutex<GroupedMetrics>>,
    shutdown: CancellationToken,
    dropped_requests: Arc<Mutex<HashMap<(String, String), u64>>>,
}

impl MetricsCollector {
    /// Creates a new metrics collector.
    pub(crate) fn new(
        receiver: tokio::sync::mpsc::Receiver<MetricsEvent>,
        dropped_requests: Arc<Mutex<HashMap<(String, String), u64>>>,
    ) -> Self {
        Self {
            receiver: Some(receiver),
            metrics: Arc::new(Mutex::new(GroupedMetrics::new())),
            shutdown: CancellationToken::new(),
            dropped_requests,
        }
    }

    /// Spawns a task that collects metrics from the receiver until the channel is
    /// closed or [`shutdown`](Self::shutdown) is called.
    ///
    /// # Arguments
    ///
    /// * `drain_ack` - Oneshot sender used to signal when the collector has finished:
    ///   the channel has been closed and all enqueued events have been processed
    ///   (or the collector was shut down and has drained remaining events). Await
    ///   the corresponding receiver before exiting to ensure metrics are fully drained.
    ///
    /// # Panics
    ///
    /// Panics if called more than once; the receiver is moved into the spawned task.
    pub(crate) fn start(
        &mut self,
        drain_ack: tokio::sync::oneshot::Sender<()>,
    ) -> tokio::task::JoinHandle<()> {
        let mut receiver =
            self.receiver.take().expect("start() can only be called once");
        let metrics = self.metrics.clone();
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(event) => {
                                Self::collect(event, &metrics).await;
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    _ = shutdown.cancelled() => {
                        break;
                    }
                }
            }
            while let Ok(event) = receiver.try_recv() {
                Self::collect(event, &metrics).await;
            }
            let _ = drain_ack.send(());
        })
    }

    /// Shuts down the metrics collector.
    pub(crate) fn shutdown(&self) {
        self.shutdown.cancel();
    }

    /// Collects metrics from an event.
    async fn collect(
        event: MetricsEvent,
        metrics: &Arc<Mutex<GroupedMetrics>>,
    ) {
        match event {
            MetricsEvent::RequestStart { scenario, request } => {
                let mut guard = metrics.lock().await;
                guard.get(&scenario, &request).inflight += 1;
            }
            MetricsEvent::RequestEnd {
                scenario,
                request,
                duration,
                succeeded,
            } => {
                let mut guard = metrics.lock().await;
                let request_metrics = guard.get(&scenario, &request);

                request_metrics.inflight -= 1;
                request_metrics.total += 1;

                if succeeded {
                    request_metrics.successful += 1;
                } else {
                    request_metrics.failed += 1;
                }

                let nanos = duration.as_nanos() as u64;
                request_metrics
                    .histogram
                    .record(nanos)
                    .map_err(|e| eprintln!("Error recording latency: {:?}", e))
                    .ok(); // TODO: handle error
            }
        }
    }

    /// Gets a snapshot of the grouped metrics.
    pub(crate) async fn get(&self) -> GroupedMetricsSnapshot {
        let mut grouped_metrics_snapshot = GroupedMetricsSnapshot::new();
        let metrics = self.metrics.lock().await;
        let dropped_requests = self.dropped_requests.lock().await;
        for (scenario, scenario_metrics) in metrics.scenarios_metrics.iter() {
            for (request, request_metrics) in
                scenario_metrics.requests_metrics.iter()
            {
                let request_metrics_snapshot =
                    grouped_metrics_snapshot.get(scenario, request);
                request_metrics_snapshot.inflight = request_metrics.inflight;
                request_metrics_snapshot.total = request_metrics.total;
                request_metrics_snapshot.successful =
                    request_metrics.successful;
                request_metrics_snapshot.failed = request_metrics.failed;
                if !request_metrics.histogram.is_empty() {
                    request_metrics_snapshot.min = Some(Duration::from_nanos(
                        request_metrics.histogram.min(),
                    ));
                    request_metrics_snapshot.max = Some(Duration::from_nanos(
                        request_metrics.histogram.max(),
                    ));
                    request_metrics_snapshot.mean =
                        Some(Duration::from_nanos(round_f64_to_u64_nanos(
                            request_metrics.histogram.mean(),
                        )));
                    request_metrics_snapshot.p50 = Some(Duration::from_nanos(
                        request_metrics.histogram.value_at_quantile(0.50),
                    ));
                    request_metrics_snapshot.p90 = Some(Duration::from_nanos(
                        request_metrics.histogram.value_at_quantile(0.90),
                    ));
                    request_metrics_snapshot.p99 = Some(Duration::from_nanos(
                        request_metrics.histogram.value_at_quantile(0.99),
                    ));
                }
            }
        }
        for ((scenario, request), count) in dropped_requests.iter() {
            let request_metrics_snapshot =
                grouped_metrics_snapshot.get(scenario, request);
            request_metrics_snapshot.dropped = *count;
        }
        grouped_metrics_snapshot
    }
}

/// A type of metrics sink.
pub enum MetricsSinkType {
    /// A sink that prints metrics to the console.
    Console,
}

/// A sink for metrics snapshots.
pub(crate) trait MetricsSink: Send + Sync {
    /// Sends a metrics snapshot.
    async fn report(&self, snapshot: &GroupedMetricsSnapshot);
}

impl MetricsSink for MetricsSinkType {
    async fn report(&self, snapshot: &GroupedMetricsSnapshot) {
        match self {
            MetricsSinkType::Console => ConsoleSink.report(snapshot).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_grouped_metrics_snapshot() {
        let dropped_requests = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = tokio::sync::mpsc::channel(10_000);
        let (drain_ack, drain_ack_receiver) = tokio::sync::oneshot::channel();
        let metrics_sender =
            MetricsSender::new(sender, dropped_requests.clone());
        let mut metrics_collector =
            MetricsCollector::new(receiver, dropped_requests.clone());
        let metrics_collector_handle = metrics_collector.start(drain_ack);

        // send some metrics events
        for i in 0..10 {
            metrics_sender.send(MetricsEvent::RequestStart {
                scenario: "test".to_string(),
                request: "test".to_string(),
            });
            metrics_sender.send(MetricsEvent::RequestEnd {
                scenario: "test".to_string(),
                request: "test".to_string(),
                duration: Duration::from_nanos(i),
                succeeded: true,
            });
        }
        drop(metrics_sender);
        drain_ack_receiver.await.unwrap();
        // get the grouped metrics snapshot
        let grouped_metrics_snapshot = metrics_collector.get().await;
        assert_eq!(grouped_metrics_snapshot.scenarios_metrics.len(), 1);
        assert_eq!(
            grouped_metrics_snapshot.scenarios_metrics["test"]
                .requests_metrics
                .len(),
            1
        );
        assert_eq!(
            grouped_metrics_snapshot.scenarios_metrics["test"]
                .requests_metrics["test"]
                .inflight,
            0
        );
        assert_eq!(
            grouped_metrics_snapshot.scenarios_metrics["test"]
                .requests_metrics["test"]
                .total,
            10
        );
        metrics_collector.shutdown();
        metrics_collector_handle.await.unwrap();
    }
}
