//! Open-loop load testing for Rust.
//!
//! Every load tester reports how fast the target responded. Granita also
//! reports *schedule delay* — the distance between when a request should have
//! been sent and when it actually was — so a run that saturated the load
//! generator is visible as such instead of being reported as server latency.
