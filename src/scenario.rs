use std::pin::Pin;

use crate::context::Context;
use crate::{Error, Request, Response};

/// A scenario is made up a sequence of steps executed in order.
pub struct Scenario {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) steps: Vec<ScenarioStep>,
}

/// A step in a scenario, either a static request or a dynamic step.
pub(crate) enum ScenarioStep {
    Static(Request),
    Dynamic(Box<dyn Step + Send + Sync>),
}

/// A step in a scenario that may produce a request from previous responses.
///
/// Implement this trait when a request depends on earlier
/// responses in the scenario.
pub trait Step: Send + Sync {
    /// Produces a request from the step.
    fn request<'a>(
        &'a self,
        context: &'a Context,
        previous_responses: &'a [Response],
    ) -> Pin<Box<dyn Future<Output = Result<Request, Error>> + Send + 'a>>;
}

impl Scenario {
    /// Creates a new scenario.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), steps: Vec::new() }
    }

    /// Adds a static request to the scenario.
    pub fn request(mut self, request: impl Into<Request>) -> Self {
        self.steps.push(ScenarioStep::Static(request.into()));
        self
    }

    /// Adds a dynamic step to the scenario.
    pub fn step(mut self, step: impl Step + 'static) -> Self {
        self.steps.push(ScenarioStep::Dynamic(Box::new(step)));
        self
    }
}
