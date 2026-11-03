# Granita

Granita is an open-loop load-testing library for Rust. It reports **schedule delay** — the distance between when a request should have been sent and when it actually was — so a user can tell whether a slow percentile came from the system under test or from the load generator.

This file is a glossary and nothing else. Every ticket, doc comment and spec uses these words with these meanings.

## Language

### Defining a test

**Plan**:
The top-level thing a user defines: a set of scenario/executor pairs, plus thresholds. A plan is a value, not an execution.
_Avoid_: Test, suite, simulation

**Scenario**:
What a virtual user does — an ordinary `async fn` over a virtual-user handle. A scenario describes behaviour and says nothing about how much or how fast.
_Avoid_: Journey, flow, task

**Step**:
A named, timed span inside a scenario, containing arbitrary code including zero or more requests. Steps do not nest: the hierarchy is exactly two levels, scenario and step.
_Avoid_: Transaction, group, task

**Request**:
One protocol-level exchange with the target. A step may contain none, one, or many.

**Executor**:
The load *model* — what the number means. Open-loop executors are shaped in arrivals per second; closed-loop executors are shaped in concurrent iterations. A scenario is paired with exactly one executor.
_Avoid_: Runner, driver, load model

**Profile**:
The shape of load over time, as a sequence of stages. The profile's unit is fixed by the executor it belongs to.
_Avoid_: Load profile, ramp, schedule

**Stage**:
One segment of a profile, such as "reach 500/s over 30 seconds" or "hold for 2 minutes". A component of a profile, never used alone.

**Feed**:
A source of per-virtual-user test data. Each virtual user is assigned one **feed record** when it is created, and reads it immutably.
_Avoid_: Data source, fixture, dataset

### Executing a test

**Run**:
One execution of a plan, producing results. A run contains one or more scenario/executor pairs executing concurrently, each independently paced.

**Virtual user**:
The thing that executes a scenario. A unit of concurrency, never a data identity — test data comes from a feed. Abbreviated **VU**.

**Iteration**:
One execution of a scenario by a virtual user, born at an **intended time**. The relationship between the two nouns is set by the executor: open-loop creates one virtual user per arrival, so VU-to-iteration is one-to-one; closed-loop virtual users run many iterations in sequence. A virtual user never runs two iterations at once.

**Session state**:
Mutable, typed state owned by a virtual user and persisting across its iterations. Distinct from the feed record, which is immutable. Under open-loop this is effectively per-iteration.

**Intended time**:
The instant at which an iteration was scheduled to begin, computed by the executor. Latency is measured from here, not from when work actually started.
_Avoid_: Scheduled time, planned time, due time

### Measuring

**Schedule delay**:
The distance between an iteration's intended time and the moment it actually started. The generator's own lateness, reported as a distribution alongside service latency and never folded into it.
_Avoid_: Lag, drift, coordinated-omission correction

**Service latency**:
How long the target took, as distinct from schedule delay. The two are always reported as separate quantities.

**Offered rate**:
The arrival rate the executor was configured to produce. Always reported paired with the achieved rate, never alone.

**Achieved rate**:
The arrival rate actually produced. A gap between offered and achieved is a generator-saturation signal.

**Measurement**:
One recorded value — this request's service latency, this iteration's schedule delay. Matches OpenTelemetry's meaning: synchronous instruments record *measurements*, whereas asynchronous instruments report *observations*. Granita's recording is synchronous, so measurement is the correct half of that pair and observation is reserved for the other.
_Avoid_: Sample, observation, data point, event

**Sample**:
Reserved, deliberately unused. In observability "sampling" means discarding a fraction of data on purpose. Granita does not do this, and the word must stay free to mean it if that ever changes.

**Metric**:
A named, typed series that measurements belong to, such as `schedule_delay`.

**Snapshot**:
A merged aggregate of measurements emitted on a timer. What sinks receive and the terminal UI renders. Maps onto OpenTelemetry data points at the exporter.
_Avoid_: Report, dump, sample

**Sink**:
A destination for snapshots — terminal UI, JSON file, OTLP endpoint.
_Avoid_: Exporter, reporter, output

**Threshold**:
A condition evaluated against a metric during a run. Thresholds never abort a run; they are displayed live, reported at the end, and reflected in the exit code.

**Saturation flag**:
A run-level signal that the load generator, rather than the target, limited the measurement. Participates in the exit code.

**Verdict**:
The end-of-run statement of which side was the bottleneck — generator-limited, target-limited, clean, ambiguous, or undetermined. The user-facing form of schedule delay.

## Deferred vocabulary

These name things that do not exist. They are defined because the invariants v1 must hold cannot be stated without them.

**Worker**:
A process that executes iterations. In v1 there is exactly one and it is the process the user ran; nothing addresses it as a worker.

**Coordinator**:
The process that would split a plan's arrival rate across workers and merge their snapshots. Does not exist.
