# Prototype Cargo build analysis

| Metadata         |                                    |
| :--              | :--                                |
| Contact | @weihanglo                         |
| Status           | Proposed                           |
| Tracking issue   | [rust-lang/goals#398] |
| Zulip channel    | N/A                                |
| [cargo] champion | @weihanglo                         |

## Summary

Prototype support for build analysis in Cargo by recording build metadata across invocations.
This will introduce unstable subcommands in `cargo report`
to explain why crates were rebuilt and to surface timing data from past builds,
helping developers diagnose and improve their build performance.

## Motivation

As Rust's popularity grows and projects increase in size and complexity,
long build times have become a growing pain point for the community and industry alike.
Understanding why builds behave the way they do and why they slow down over time
is critical to improving the developer experience of Rust and Cargo.

### The status quo

Other than caching build artifacts and future-incompatibility reports,
Cargo does not persist information about previous builds.
While the [`--timings`] flag provides per-build data on how long each unit takes to compile,
this information:

* You have to know you will care about a timings report and can't look it up afterwards
* Exists primarily in HTML form, which is not suited for machine analysis

Additionally, Cargo does not track:

* Build cache effectiveness and rebuild reasons
* Invocation metadata (e.g., CLI flags, profiles, environment variables)
* System resources usage (e.g., memory and CPU)

This limits our ability to:

* Understand real-world build behavior over time
* Explain why crates were rebuilt between builds
* Experiment with performance optimizations, such as adaptive scheduling

[`--timings`]: https://doc.rust-lang.org/nightly/cargo/reference/timings.html

### The next 6 months

Cargo will provide an opt-in unstable configuration option to collect the following data:

* The existing metrics collected by the `--timings` flag
* Rebuild reasons and related information
* CLI arguments for each build

Each data record will be associated with a build identifier,
such as `CARGO_RUN_ID`,
to make it possible to link related data within the same Cargo invocation.

Two new unstable commands in `cargo report` will be introduced (command name TBD):

* `cargo report rebuild-reasons`:
  Show which crates were rebuilt for a specific Cargo run and why.
* `cargo report timing`:
  Display timing data for a specific build, including per-crate compile times.

During the prototyping phase,
the data may be stored as JSON blobs in a simple database format, such as SQLite.
This approach allows schema evolution without committing to a stable format.

### The "shiny future" we are working towards

* Experiment with adaptive scheduling to reduce critical path length in builds
* Extend `cargo report` with richer insights, such as:
  * Highlighting frequently rebuilt crates and their triggers
  * Summarizing build metrics and offering actionable suggestions
* Enable external tooling to:
  * Analyze historical trends and identify performance bottlenecks
  * Provide live insights into slow build steps during development
* Record additional build metrics to unlock more future capabilities and analysis,
  such as build replay for debugging and CI reproducibility

## Design axioms

* The changes to Cargo should not impede the development of Cargo
* Metric collection must avoid introducing privacy or performance concerns
* No user-facing stability guarantees during the prototyping phase
* Data collection is opt-in and disabled by default

## Ownership and team asks

| Task                              | Owner(s) or team(s)                | Notes |
|-----------------------------------|------------------------------------|-------|
| Discussion and moral support      | ![Team][] [cargo]                  |       |
| Implementation                    | ![Help wanted][] @weihanglo        |       |
| Standard reviews                  | ![Team][] [cargo]                  |       |
| Author call for testing blog post | @weihanglo                         |       |

### Definitions

For definitions for terms used above, see the [About > Team Asks](https://goals.rust-lang.org/about/team_asks.html) page.

* *Discussion and moral support* is the lowest level offering, basically committing the team to nothing but good vibes and general support for this endeavor.
* *Author RFC* and *Implementation* means actually writing the code, document, whatever.
* *Design meeting* means holding a synchronous meeting to review a proposal and provide feedback (no decision expected).
* *RFC decisions* means reviewing an RFC and deciding whether to accept.
* *Org decisions* means reaching a decision on an organizational or policy matter.
* *Secondary review* of an RFC means that the team is "tangentially" involved in the RFC and should be expected to briefly review.
* *Stabilizations* means reviewing a stabilization and report and deciding whether to stabilize.
* *Standard reviews* refers to reviews for PRs against the repository; these PRs are not expected to be unduly large or complicated.
* *Prioritized nominations* refers to prioritized lang-team response to nominated issues, with the expectation that there will be *some* response from the next weekly triage meeting.
* *Dedicated review* means identifying an individual (or group of individuals) who will review the changes, as they're expected to require significant context.
* Other kinds of decisions:
    * [Lang team experiments](https://lang-team.rust-lang.org/how_to/experiment.html) are used to add nightly features that do not yet have an RFC. They are limited to trusted contributors and are used to resolve design details such that an RFC can be written.
    * Compiler [Major Change Proposal (MCP)](https://forge.rust-lang.org/compiler/mcp.html) is used to propose a 'larger than average' change and get feedback from the compiler team.
    * Library [API Change Proposal (ACP)](https://std-dev-guide.rust-lang.org/development/feature-lifecycle.html) describes a change to the standard library.

## Frequently asked questions

### Is this telemetry?

No. This is not.
Cargo follows the principles described in <http://esteban.kuber.com.ar/rustc-metrics.html>
and does not send data anywhere.
All metrics are stored locally and remain under the user's control.
