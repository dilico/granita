use std::pin::Pin;

use crate::Error;
use crate::context::Context;

/// A scenario for a load test.
pub(crate) struct Scenario {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) func: Box<dyn ScenarioFnTrait + Send + Sync>,
}

/// A trait for scenario functions.
pub(crate) trait ScenarioFnTrait {
    fn call<'a>(
        &'a self,
        context: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
}

/// A function that represents a scenario.
///
/// This is a type alias for convenience. The function takes a context and returns
/// a pinned, boxed future that resolves to a result.
///
/// # Arguments
///
/// * `context` - The context for the scenario.
///
/// # Returns
///
/// * A pinned, boxed future that resolves to `Ok(())` on success or `Err(error)` on failure.
pub type ScenarioFn = for<'a> fn(
    &'a Context,
) -> Pin<
    Box<dyn Future<Output = Result<(), Error>> + Send + 'a>,
>;

/// A wrapper that implements ScenarioFnTrait for any function that returns a boxed future.
pub(crate) struct ScenarioFnWrapper<F> {
    pub(crate) func: F,
}

impl<F> ScenarioFnTrait for ScenarioFnWrapper<F>
where
    F: for<'a> Fn(
            &'a Context,
        ) -> Pin<
            Box<dyn Future<Output = Result<(), Error>> + Send + 'a>,
        > + Send
        + Sync
        + 'static,
{
    fn call<'a>(
        &'a self,
        context: &'a Context,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        (self.func)(context)
    }
}

/// A macro to create a scenario function.
///
/// # Arguments
///
/// * `ctx` - The context for the scenario.
/// * `body` - The body of the scenario.
///
/// # Returns
///
/// * A scenario function that takes a context and returns a pinned, boxed future that resolves to a result.
///
/// # Example
///
/// ```
/// use granita::{Granita, scenario_fn};
///
/// let granita = Granita::new()
///     .scenario("test", scenario_fn!(|ctx| { Ok(()) }));
/// ```
#[macro_export]
macro_rules! scenario_fn {
    (|$ctx:ident| $body: block) => {
        |$ctx: &$crate::context::Context| Box::pin(async $body)
    };
}
