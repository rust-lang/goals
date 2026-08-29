# Continue resolving `cargo-semver-checks` blockers for merging into cargo

| Metadata         |                                    |
|:-----------------|------------------------------------|
| Contact | @obi1kenobi                        |
| Status           | Accepted                           |
| Tracking issue   | [rust-lang/goals#104] |
| Zulip channel    | N/A                                |
## Summary

Design and implement `cargo-semver-checks` functionality that lies on the critical path for merging the tool into cargo itself. Continues the work of [the 2024h2 goal][2024h2-goal].

[2024h2-goal]: https://goals.rust-lang.org/2024h2/cargo-semver-checks.html

## Motivation

Cargo assumes that all packages adhere to semantic versioning (SemVer).
However, SemVer adherence is quite hard in practice: [research shows][semver-study] that accidental SemVer violations are relatively common (lower-bound: in 3% of releases) and happen to Rustaceans of all skill levels.
Given the significant complexity of the Rust SemVer rules, improvements here require better tooling.

`cargo-semver-checks` is a linter for semantic versioning (SemVer) in Rust.
It is broadly adopted by the Rust community, and the [cargo] team has expressed interest in merging it into cargo itself as part of the existing `cargo publish` workflow.
By default, `cargo publish` would require SemVer compliance, but offer a flag (analogous to the `--allow-dirty` flag for uncommitted changes) to override the SemVer check and proceed with publishing anyway.

The [cargo] team has identified [a set of milestones and blockers][merge-blockers] that must be resolved before `cargo-semver-checks` can be integrated into the `cargo publish` workflow.
Our goal here is to make steady progress toward resolving them.

[semver-study]: https://predr.ag/blog/semver-violations-are-common-better-tooling-is-the-answer/
[merge-blockers]: https://github.com/obi1kenobi/cargo-semver-checks/issues/61

### The status quo after the 2024h2 goal

As part of [the 2024h2 goal work][2024h2-tracking], support for cargo manifest linting was merged into `cargo-semver-checks`.
This lifted [one of the blockers][merge-blockers] blocker for SemVer-checking as part of `cargo publish`.

Work is still required in two major areas:
- Checking of cross-crate items
- SemVer linting of type information

Some work in each of these areas [already happened in the 2024h2 goal][2024h2-tracking]:
- The manifest linting work [required a significant refactor][major-refactor] of the tool's data-handling infrastructure. As part of that major refactor, we were able to also create "API space" for a future addition of cross-crate information.
- The [compiler team MCP][compiler-mcp] required to expose cross-crate information to rustdoc was merged, and together with T-rustdoc, we now have a plan for exposing that information to `cargo-semver-checks`.
- We have implemented a partial schema that makes available a limited subset of type information around generic parameters and trait bounds. It's sufficient to power a set of new lints, though it isn't comprehensive yet.

Fully resolving the [blockers][merge-blockers] is likely a 12-24 month undertaking, and beyond the scope of this goal on its own.
Instead, this goal proposes to accomplish intermediate steps that create immediate value for users and derisk the overall endeavor, while needing only "moral support" from the [cargo] team as its only requirement.

[2024h2-tracking]: https://github.com/rust-lang/goals/issues/104
[major-refactor]: https://github.com/obi1kenobi/cargo-semver-checks/pull/1001
[compiler-mcp]: https://github.com/rust-lang/compiler-team/issues/635

#### Checking of cross-crate items

_This section is background information and is unchanged from [the 2024h2 goal][2024h2-goal]._

Currently, `cargo-semver-checks` performs linting by only using the rustdoc JSON of the target package being checked.
However, the public API of a package may expose items from other crates.
Since rustdoc no longer inlines the definitions of such foreign items into the JSON of the crate whose public API relies on them, `cargo-semver-checks` [cannot see or analyze them][cross-crate-items].

This causes a massive number of false-positives ("breakage reported incorrectly") and false-negatives ("lint for issue X fails to spot an instance of issue X").
In excess of 90% of real-world false-positives are traceable back to a cross-crate item, as measured by our [SemVer study][semver-study]!

For example, the following change is not breaking but `cargo-semver-checks` will incorrectly report it as breaking:
```rust
// previous release:
pub fn example() {}

// in the new release, imagine this function moved to `another_crate`:
pub use another_crate::example;
```
This is because the rustdoc JSON that `cargo-semver-checks` sees indeed *does not contain* a function named `example`.
Currently, `cargo-semver-checks` is incapable of following the cross-crate connection to `another_crate`, generating its rustdoc JSON, and continuing its analysis there.

Resolving this limitation will require changes to how `cargo-semver-checks` generates and handles rustdoc JSON, since the set of required rustdoc JSON files will no longer be fully known ahead of time.
It will also require CLI changes in the same area as the changes required to support manifest linting.

While there may be other challenges on rustc and rustdoc's side before this feature could be fully implemented, we consider those out of scope here since there are [parallel efforts to resolve them][parallel-efforts].
The goal here is for `cargo-semver-checks` to have its own story straight and do the best it can.

[cross-crate-items]: https://github.com/obi1kenobi/cargo-semver-checks/issues/638
[parallel-efforts]: https://github.com/rust-lang/compiler-team/issues/635

#### SemVer linting of type information

_This section is background information and is unchanged from [the 2024h2 goal][2024h2-goal]._

In general, at the moment `cargo-semver-checks` lints cannot represent or examine type information.
For example, the following change is breaking but `cargo-semver-checks` will not detect or report it:
```rust
// previous release:
pub fn example(value: String) {}

// new release:
pub fn example(value: i64) {}
```
Analogous breaking changes to function return values, struct fields, and associated types would also be missed by `cargo-semver-checks` today.

The main difficulty here lies with the expressiveness of the Rust type system. For example, none of the following changes are breaking:
```rust
// previous release:
pub fn example(value: String) {}

// new release:
pub fn example(value: impl Into<String>) {}

// subsequent release:
pub fn example<S: Into<String>>(value: S) {}
```
Similar challenges exist with lifetimes, variance, trait solving, `async fn` versus `fn() -> impl Future`, etc.

While some promising preliminary work has been done toward resolving this challenge, more in-depth design work is necessary to determine the best path forward.

### The next 6 months

- Prototype cross-crate linting using manual workarounds for the current rustc and rustdoc blockers. This will allow us to roll out a full solution relatively quickly after the rustc and rustdoc blockers are resolved.
- Expose data on generic types, lifetimes, functions, methods, and bounds in sufficient granularity for linting.
- Determine how to handle special cases, such as changes to impls or bounds involving `'static`, `?Sized`, `dyn Trait` etc.
- Improve sealed trait analysis to account for `#[doc(hidden)]` items, resolving many false-positives.

### The "shiny future" we are working towards

_This section is unchanged from [the 2024h2 goal][2024h2-goal]._

Accidentally publishing SemVer violations that break the ecosystem is never fun for anyone involved.

From a user perspective, we want a fearless `cargo update`: one's project should never be broken by updating dependences without changing major versions.

From a maintainer perspective, we want a fearless `cargo publish`: we want to prevent breakage, not to find out about it when a frustrated user opens a GitHub issue. Just like cargo flags uncommitted changes in the publish flow, it should also _quickly_ and _accurately_ flag breaking changes in non-major releases. Then the maintainer may choose to release a major version instead, or acknowledge and explicitly override the check to proceed with publishing as-is.

To accomplish this, `cargo-semver-checks` needs the ability to express more kinds of lints (including manifest and type-based ones), eliminate false-positives, and stabilize its public interfaces (e.g. the CLI). At that point, we'll have lifted the [main merge-blockers][merge-blockers] and we can consider making it a first-party component of cargo itself.

## Ownership and team asks

**Owner:** @obi1kenobi, as maintainer of `cargo-semver-checks`

I (@obi1kenobi) will be working on this effort. The only other resource request would be occasional discussions and moral support from the [cargo] and [rustdoc] teams, of which I already have the privilege as maintainer of a popular cargo plugin that makes extensive use of rustdoc JSON.

| Task                                               | Owner(s) or team(s)         | Notes |
| -------------------------------------------------- | --------------------------- | ----- |
| Prototype cross-crate linting using workarounds    | @obi1kenobi                 |       |
| Allow linting generic types, lifetimes, bounds     | @obi1kenobi                 |       |
| Handle "special cases" like `'static` and `?Sized` | @obi1kenobi                 |       |
| Handle `#[doc(hidden)]` in sealed trait analysis   | @obi1kenobi                 |       |
| Discussion and moral support                       | ![Team][] [cargo] [rustdoc] |       |

## Frequently asked questions

_This section is unchanged from [the 2024h2 goal][2024h2-goal]._

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
