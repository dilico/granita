/// A load profile is a set of rules for how to load test a scenario.
pub enum LoadProfile {
    /// Run the scenario once.
    RunOnce,
    /// Run the scenario for a constant number of iterations.
    ConstantIterations {
        /// The number of virtual users to run the scenario with.
        vus: u64,
        /// The number of iterations to run the scenario for.
        iterations: u64,
    },
}
