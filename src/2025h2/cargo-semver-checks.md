# Continue resolving `cargo-semver-checks` blockers for merging into cargo

| Metadata         |                                    |
|:-----------------|------------------------------------|
| Contact | @obi1kenobi                        |
| Status           | Proposed                           |
| Tracking issue   | [rust-lang/goals#104] |
| Zulip channel    | N/A                                |
| [cargo] champion | @epage |
| [rustdoc] champion | @adotinthevoid |

## Summary

Design and implement `cargo-semver-checks` functionality that lies on the critical path for merging the tool into cargo itself. Continues the work of [the 2025h1 goal][2025h1-goal].

[2024h2-goal]: https://goals.rust-lang.org/2024h2/cargo-semver-checks.html
[2025h1-goal]: https://goals.rust-lang.org/2025h1/cargo-semver-checks.html

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

### The status quo after the 2025h1 goal period

The 2025h1 goal targeted work in two major areas:
- Checking of cross-crate items
- SemVer linting of type information

While meaningful progress was achieved, significant work remains to be done.

`'static` bounds were discovered to be able to be _implied_, meaning that the bound is not syntactically present at the public API definition site, but is nevertheless part of the public API via being implied by another bound (or by a bound implied by such a bound, etc.).
Conversely, `?Sized` was discovered to possibly not apply even when syntactically present, as an implied bound elsewhere may imply `Sized` and take precedence over `?Sized`.
SemVer linting requires precise—not just syntactic—information in both of these cases.
Failure to obtain precise information would lead to both false-positives and false-negatives, which would be an unacceptable user experience.
Rustdoc JSON today does not include such precise information; the addition of such information is currently under discussion with the [rustdoc] team.

Outside of these special cases of type-checking lints, some progress was made on a concrete plan for more general type-checking lints.
The current plan is to make use of Rust's only stable API: "please compile this program for me."
Whenever types in any public API location might appear to have changed, `cargo-semver-checks` would generate a witness program and check it via `cargo check`, such that compilation would _only_ succeed if the type change is backward-compatible.
This is an extension of the same technique we used several years ago in our survey of SemVer compliance and breakage in the Rust ecosystem.
One of Rust's Google Summer of Code participants is working toward making witness generation work end-to-end, and we're thrilled to be working together!

As part of the All Hands at RustWeek, we also made progress toward enabling cross-crate linting.
We identified a set of changes in Rust tooling that, when implemented, will result in `cargo-semver-checks` being able to uniquely determine which crate and version a given item came from.
While all stakeholders are aligned on the idea and have reasonable confidence that it will work as planned, much more implementation work remains to be done and it's possible additional obstacles may be identified along the way.

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

- Expose precise information regarding `'static` and `?Sized`, then make use of it in lints.
- Audit our current set of lints related to trait and lifetime bounds, aiming to create a comprehensive list of what lints we still need to add.
- Support our GSoC contributor in building witness program generation and type-checking infrastructure.
- Support the [rustdoc] team in exposing additional information we require, by contributing ideas and pull requests as needed while seeking to minimize the review and maintenance burden thus applied.

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
| Expose precise `'static` and `?Sized` info         | @obi1kenobi  |       |
| Lints for `'static` and `?Sized`                   | @obi1kenobi                 |       |
| Audit lints for lifetime and trait bounds          | @obi1kenobi                 |       |
| Support our GSoC contributor's work on witness programs and type-checking infra | @obi1kenobi |       |
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
