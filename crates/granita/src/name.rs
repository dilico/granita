use std::fmt;

/// The name of a step, used as its label in every report.
///
/// A step is a named, timed span inside a scenario. Each distinct name
/// becomes its own metric, so the set of names must be fixed.
///
/// The name is a `&'static str`: a string literal is accepted, a string
/// built at runtime is not. A name built with `format!` would create a new
/// metric for every value; this type makes that a compile error.
///
/// To build a name from runtime data, such as a configuration file read at
/// startup, `String::leak` produces a `&'static str`. Do this once per name,
/// never inside a scenario: every call leaks memory.
///
/// # Examples
///
/// ```
/// use granita::StepName;
///
/// let login = StepName::new("login");
/// assert_eq!(login.as_str(), "login");
/// ```
///
/// A name built at runtime is rejected, even when borrowed as a `&str`:
///
/// ```compile_fail,E0716
/// use granita::StepName;
///
/// StepName::new(&format!("x"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepName {
    name: &'static str,
}

impl StepName {
    /// Creates a step name.
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
    /// Returns the step name as a string slice.
    pub const fn as_str(&self) -> &'static str {
        self.name
    }
}

impl fmt::Display for StepName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.name, f)
    }
}

/// The name of a scenario, used as its label in every report.
///
/// A scenario is what a virtual user does: an ordinary `async fn` over a
/// virtual-user handle. Like [`StepName`], the name must be a `&'static str`.
///
/// # Examples
///
/// ```
/// use granita::ScenarioName;
///
/// let checkout = ScenarioName::new("checkout");
/// assert_eq!(checkout.as_str(), "checkout");
/// ```
///
/// A name built at runtime is rejected, even when borrowed as a `&str`:
///
/// ```compile_fail,E0716
/// use granita::ScenarioName;
///
/// ScenarioName::new(&format!("x"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScenarioName {
    name: &'static str,
}

impl ScenarioName {
    /// Creates a scenario name.
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
    /// Returns the scenario name as a string slice.
    pub const fn as_str(&self) -> &'static str {
        self.name
    }
}

impl fmt::Display for ScenarioName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.name, f)
    }
}
