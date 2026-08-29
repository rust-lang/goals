# Rework Cargo Build Dir Layout

| Metadata         |                                                                                  |
| :--              | :--                                                                              |
| Contact | @ranger-ross                                                                     |
| Status           | Proposed                                                                         |
| Tracking issue   | [rust-lang/goals#401]                                               |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |
| [cargo] champion | @weihanglo                                                                       |

## Summary

Rework the Cargo build directory layout to have smaller self contained "units"
## Motivation

Reworking the build directory layout into units will allow us to work to greater Cargo goals.

Notably

* Fine grain locking within the build directory
  * Reducing build dir lock contention between Cargo and Rust Analyzer
* GC of target directories
* A cross workspace shared build cache

### The status quo

When Cargo performs a build, it will build the package you requested and all
dependencies individually, linking them in the end.
These build results (intermediate build artifacts) are stored in the Cargo [build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html). (commonly referred to as build-dir)

The build cache organizes files in a way that is not easily broken down into smaller units.
Due to this Cargo will lock the entire build cache when operating to prevent multiple Cargo processes from interfering with each other.
This locking is commonly felt by users when Rust Analyzer triggers a `cargo check` while the user is attempting to build the project with `cargo build`, resulting in the build needing to wait for cargo check to finish.

The build cache layout is also makes caching in CI difficult as the intermediate build artifacts for each crate are scattered across many subdirectories.
Users will often cache the entire `target` directory or attempt selectively cache build cache internals by making assumptions about Cargo implementation details.

The build cache is shared for projects in a Cargo workspace, however some users share the build cache by setting a shared `CARGO_TARGET_DIR`.
This comes with some limitations like running `cargo clean` will remove the build cache for all workspaces.

For more information about the pain points with the status quo see the following issues:
* [Re-organize build-dir by package + hash, rather than artifact type cargo#15010](https://github.com/rust-lang/cargo/issues/15010)
* [cargo ./target fills with outdated artifacts as toolchains are updated/changed cargo#5026](https://github.com/rust-lang/cargo/issues/5026)
* [More granular locking in cargo_rustc cargo#4282](https://github.com/rust-lang/cargo/issues/4282)
* [Per-user compiled artifact cache cargo#5931](https://github.com/rust-lang/cargo/issues/5931)

### The next 6 months

* Rework the build directory layout so that we have more easily cacheable/lockable "units" (rust-lang/cargo#15010)
* Rework the build directory locking to avoid locking the entire build directory.

### The "shiny future" we are working towards

The build directory layout is reorganized in a way that makes fine grain caching and locking easier.
Tools that leverage Cargo (like Rust Analyzer) have reduced lock contention and generally are able to operate in parallel on a shared build cache.

A new "user wide" cache is created as a first class solution for sharing build cache artifacts across workspaces.

The cache lookup will be extended with plugins to read and/or write to different sources. Open source projects and companies can have their CI read from and write to their cache. Individuals who trust the CI can then configure their plugin to read from the CI cache.

A cooperating CI service could provide their own plugin that, instead of caching everything used in the last job and unpacking it in the next, their plugin could download only the entries that will be needed for the current build (e.g. say a dependency changed) and only upload the cache entries that were freshly built. Fine grain caching like this would save the CI service on bandwidth, storage, and the compute time from copying, decompressing, and compressing the cache. Users would have faster CI time and save money on their CI service, minus any induced demand that faster builds creates.
## Ownership and team asks

| Task                         | Owner(s) or team(s) | Notes |
|------------------------------|---------------------|-------|
| Standard reviews             | ![Team][] [cargo]   |       |
| Implementation               | @ranger-ross        |       |
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

### Why not pre-built packages?

Pre-built packages requires guessing
- CPU Architecture
- Feature flags
- RUSTFLAGS
- Dependency versions

If there are any mismatches there, then the pre-built package can't be used.

A build cache can be populated with pre-built packages and react to the unique circumstances of the user.

### Why not sccache?

Tools like sccache try to infer inputs for hashing a cache key from command-line arguments.
This has us reusing the extra knowledge Cargo has to get more accurate cache key generation.
