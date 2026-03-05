use std::pin::Pin;

use crate::context::Context;
use crate::{Error, Request, Response};

/// A scenario is made up a sequence of steps executed in order.
pub struct Scenario {
    pub(crate) name: String,
    pub(crate) steps: Vec<ScenarioStep>,
}

/// A step in a scenario.
pub(crate) struct ScenarioStep {
    pub(crate) name: String,
    pub(crate) request: ScenarioStepRequest,
}

/// A request in a scenario step, either a static request or a dynamic step.
pub(crate) enum ScenarioStepRequest {
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
    pub fn request(
        mut self,
        name: impl Into<String>,
        request: impl Into<Request>,
    ) -> Self {
        self.steps.push(ScenarioStep {
            name: name.into(),
            request: ScenarioStepRequest::Static(request.into()),
        });
        self
    }

    /// Adds a dynamic step to the scenario.
    pub fn step(
        mut self,
        name: impl Into<String>,
        step: impl Step + 'static,
    ) -> Self {
        self.steps.push(ScenarioStep {
            name: name.into(),
            request: ScenarioStepRequest::Dynamic(Box::new(step)),
        });
        self
    }
}
