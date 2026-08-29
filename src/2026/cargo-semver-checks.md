# Continue resolving `cargo-semver-checks` blockers for merging into cargo

| Metadata           |                                    |
|:-------------------|------------------------------------|
| Contact   | @obi1kenobi                        |
| Status             | Accepted                           |
| Tracking issue     | [rust-lang/goals#104] |
| Zulip channel      | N/A                                |
| [cargo] champion   | @epage                             |
| [rustdoc] champion | @adotinthevoid                     |

## Summary

Design and implement `cargo-semver-checks` functionality that lies on the critical path for merging the tool into cargo itself. Continues the work of [the 2025h2 goal][2025h2-goal].

[2025h2-goal]: https://goals.rust-lang.org/2025h2/cargo-semver-checks.html

## Motivation

Cargo assumes that all packages adhere to semantic versioning (SemVer).
However, SemVer adherence is quite hard in practice: [research shows][semver-study] that accidental SemVer violations are relatively common (lower-bound: in 3% of releases) and happen to Rustaceans of all skill levels.
Given the significant complexity of the Rust SemVer rules, improvements here require better tooling.

`cargo-semver-checks` is a linter for semantic versioning (SemVer) in Rust.
It is broadly adopted by the Rust community, and the [cargo] team has expressed interest in merging it into cargo itself as part of the existing `cargo publish` workflow.
By default, `cargo publish` would require SemVer compliance, but offer a flag (analogous to the `--allow-dirty` flag for uncommitted changes) to override the SemVer check and proceed with publishing anyway.

The [cargo] team has identified [a set of milestones and blockers][merge-blockers] that must be resolved before `cargo-semver-checks` can be integrated into the `cargo publish` workflow.
Our goal here is to make steady progress toward resolving them.

From a user perspective, we want a fearless `cargo update`: one's project should never be broken by updating dependences without changing major versions.

From a maintainer perspective, we want a fearless `cargo publish`: we want to prevent breakage, not to find out about it when a frustrated user opens a GitHub issue. Just like cargo flags uncommitted changes in the publish flow, it should also _quickly_ and _accurately_ flag breaking changes in non-major releases. Then the maintainer may choose to release a major version instead, or acknowledge and explicitly override the check to proceed with publishing as-is.

To accomplish this, `cargo-semver-checks` needs the ability to express more kinds of lints (including manifest and type-based ones), eliminate false-positives, and stabilize its public interfaces (e.g. the CLI). At that point, we'll have lifted the [main merge-blockers][merge-blockers] and we can consider making it a first-party component of cargo itself.

[semver-study]: https://predr.ag/blog/semver-violations-are-common-better-tooling-is-the-answer/
[merge-blockers]: https://github.com/obi1kenobi/cargo-semver-checks/issues/61

### The status quo

`cargo-semver-checks` currently ships with 245 lints, and is effective at catching many forms of breakage — both in [crate manifests](https://predr.ag/blog/breakage-in-the-cargo-toml-how-rust-package-features-work/) and also inside crates' source code.

However, many more lints remain to be written, and they will require additional infrastructure in both rustdoc JSON and in `cargo-semver-checks` itself, as we describe below.

For a detailed look at the status quo, we recommend checking out [the most recent `cargo-semver-checks` annual summary](https://predr.ag/blog/cargo-semver-checks-2025-year-in-review/)


### What we propose to do about it

The following are the largest remaining blockers for merging `cargo-semver-checks` in Cargo:

#### Performing type-checking in lints

<https://github.com/obi1kenobi/cargo-semver-checks/issues/149>

This lets us catch changes like: `pub fn example(x: i64) {}` becoming `pub fn example(x: String) {}`.

This is our most commonly requested feature today, and will resolve the largest remaining class of false-negative (lint should fire, but doesn't) outcomes!

We plan to accomplish this in two steps:
- Expose additional information in rustdoc JSON to make it possible to reliably observe that something about a type has changed. This will enable some lints by itself, but is not sufficient in all cases: for example, changing `impl Display` to `String` or vice versa isn't always breaking.
- Build out infrastructure in `cargo-semver-checks` to make it possible to generate "witness" programs: ones on which `cargo check` can be executed in order to determine whether a changed type caused breakage or not. This will allow us to rely on `rustc` to be the arbiter of breakage, instead of requiring us to reimplement the Rust type checker, trait solver, borrow checker, etc. Some of the infrastructure here was already built [as part of GSoC 2025](https://blog.rust-lang.org/2025/11/18/gsoc-2025-results/#enable-witness-generation-in-cargo-semver-checks), and we expect to continue building on top of that foundation.

#### Linting across crate boundaries

<https://github.com/obi1kenobi/cargo-semver-checks/issues/638>

Linting across crate boundaries with high reliability and acceptable performance, so that for example, we can correctly handle cross-crate item re-exports. This will resolve the largest remaining class of false-positive bugs (a lint fires where it shouldn't) in our linting system.

Completing this will require close cooperation with T-rustdoc, to enable rustdoc JSON files to be reliably connected to each other. This work already began in 2025 with @adotinthevoid leading it from the rustdoc side.

#### Participate Google Summer of Code (GSoC)

The main blocker towards progress is funding and development capacity. `cargo-semver-checks` has successfully participated in GSoC before and we plan to do it again.

### Work items over the next year

**Owner:** @obi1kenobi, as maintainer of `cargo-semver-checks`

I (@obi1kenobi) will be working on this effort. The only other resource request would be occasional discussions and moral support from the [cargo] and [rustdoc] teams, of which I already have the privilege as maintainer of a popular cargo plugin that makes extensive use of rustdoc JSON.

| Task                                      | Owner(s)    | Notes |
|-------------------------------------------|-------------|-------|
| Audit lint contributions                  | @obi1kenobi |       |
| Mentor GSoC participants                  | @obi1kenobi |       |
| Implement type-checking in lints          |             |       |
| Implement linting across crate boundaries |             |       |


## Team asks


| Team      | Support level | Notes                        |
|-----------|---------------|------------------------------|
| [cargo]   | Small         | Discussion and moral support |
| [rustdoc] | Small         | Discussion and moral support |

## Frequently asked questions

_This section is unchanged from [the 2024h2 goal][2024h2-goal]._

[2024h2-goal]: https://goals.rust-lang.org/2024h2/cargo-semver-checks.html


### Why not use semverver instead?

[Semverver][semverver] is a prior attempt at enforcing SemVer compliance, but has been deprecated and is no longer developed or maintained.
It relied on compiler-internal APIs, which are much more unstable than rustdoc JSON and required much more maintenance to "keep the lights on."
This also meant that semverver required users to install a specific nightly versions that were known to be compatible with their version of semverver.

While `cargo-semver-checks` relies on rustdoc JSON which is also an unstable nightly-only interface, its changes are much less frequent and less severe.
By using the [Trustfall query engine][trustfall], `cargo-semver-checks` can simultaneously support a range of rustdoc JSON formats (and therefore Rust versions) within the same tool.
On the maintenance side, `cargo-semver-checks` lints are written in a declarative manner that is oblivious to the details of the underlying data format, and do not need to be updated when the rustdoc JSON format changes.
This makes maintenance much easier: updating to a new rustdoc JSON format usually requires just a few lines of code, instead of "a few lines of code apiece in each of hundreds of lints."

[semverver]: https://github.com/rust-lang/rust-semverver
[trustfall]: https://github.com/obi1kenobi/trustfall
