use std::pin::Pin;

use crate::Error;
use crate::context::Context;
use crate::scenario::{Scenario, ScenarioFnWrapper};

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
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the scenario.
    /// * `scenario` - The scenario function to execute. This function takes a context
    ///   and returns a future that resolves to a result. The future will be automatically
    ///   boxed and pinned when added to the builder.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use granita::{Granita, Error, Request};
    /// use granita::request::HttpRequest;
    /// use granita::scenario_fn;
    ///
    /// let granita = Granita::new().scenario("my_scenario", scenario_fn!(|ctx| {
    ///         let request = HttpRequest::get("https://example.com")
    ///             .build()
    ///             .map_err(|_| Error::Configuration("Invalid URL".into()))?;
    ///         ctx.send(Request::Http(request)).await?;
    ///         Ok(())
    ///     }));
    /// ```
    pub fn scenario<F>(mut self, name: impl Into<String>, scenario: F) -> Self
    where
        F: for<'a> Fn(
                &'a Context,
            ) -> Pin<
                Box<dyn Future<Output = Result<(), Error>> + Send + 'a>,
            > + Send
            + Sync
            + 'static,
    {
        self.scenarios.push(Scenario {
            name: name.into(),
            func: Box::new(ScenarioFnWrapper { func: scenario }),
        });
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
        for scenario in self.scenarios {
            scenario.func.call(&context).await?;
        }
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
    use crate::scenario_fn;

    use super::*;

    #[tokio::test]
    async fn scenario_adds_scenario() {
        let granita =
            Granita::new().scenario("test", scenario_fn!(|_ctx| { Ok(()) }));
        assert_eq!(granita.scenarios.len(), 1);
        assert_eq!(granita.scenarios[0].name, "test");
    }

    #[tokio::test]
    async fn scenario_adds_multiple_scenarios() {
        let granita = Granita::new()
            .scenario("test1", scenario_fn!(|_ctx| { Ok(()) }))
            .scenario("test2", scenario_fn!(|_ctx| { Ok(()) }));
        assert_eq!(granita.scenarios.len(), 2);
        assert_eq!(granita.scenarios[0].name, "test1");
        assert_eq!(granita.scenarios[1].name, "test2");
    }
}
