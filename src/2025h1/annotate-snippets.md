# Use annotate-snippets for rustc diagnostic output

| Metadata         |                                    |
|:-----------------|------------------------------------|
| Contact | @Muscraft                          |
| Status           | Accepted                           |
| Tracking issue   | [rust-lang/goals#123] |
| Zulip channel    | N/A                                |
## Summary

Switch to annotate-snippets for rendering rustc's output, with no loss of functionality or visual regressions.

## Motivation

Cargo has been [adding its own linting system][cargo-lints], where it has been using annotate-snippets to try and match Rust's output. This has led to duplicate code between the two, increasing the overall maintenance load. Having one renderer that produces Rust-like diagnostics will make it so there is a consistent style between Rust and Cargo, as well as any other tools with similar requirements like miri, and should lower the overall maintenance burden by rallying behind a single unified solution.

### The status quo

Currently rustc has its own Emitter that encodes the theming properties of compiler diagnostics. It has handle all of the intricacies of terminal support (optional color, terminal width querying and adapting of output), layout (span and label rendering logic), and the presentation of different levels of information. Any tool that wants to approximate rustc's output for their own purposes, needs to use a third-party tool that diverges from rustc's output, like annotate-snippets or miette. Any improvements or bugfixes contributed to those libraries are not propagated back to rustc. Because the emitter is part of the rustc codebase, the barrier to entry for new contributors is artificially kept high than it otherwise would be.

annotate-snippets is already part of the rustc codebase, but it is disabled by default, doesn't have extensive testing and there's no way of enabling this output through cargo, which limits how many users can actually make use of it.

### The next 6 months

- annotate-snippets rendered output reaches full parity (modulo reasonable non-significant divergences) with rustc's output
- A call for testing is made to the community to gather feedback on annotate-snippets

### The "shiny future" we are working towards

The outputs of rustc and cargo are fully using annotate-snippets, with no regressions to the rendered output. annotate-snippets grows its feature set, like support for more advanced rendering formats or displaying diagnostics with more than ASCII-art, independently of the compiler development cycle.

## Design axioms

- **Match rustc's output**: The output of annotate-snippets should match rustc, modulo reasonable non-significant divergences
- **Works for Cargo (and other tools)**: annotate-snippets is meant to be used by any project that would like "Rust-style" output, so it should be designed to work with any project, not just rustc.
[da]: https://goals.rust-lang.org/about/design_axioms.html

## Ownership and team asks

**Owner:** @estebank, @Muscraft

### Reach output parity of rustc/annotate-snippets

| Task                              | Owner(s) or team(s) | Notes |
|-----------------------------------|---------------------|-------|
| add suggestions                   | @Muscraft           |       |
| Port a subset of rustc's UI tests | @Muscraft           |       |
| address divergences               | @Muscraft           |       |

### Initial use of annotate-snippets

| Task                                       | Owner(s) or team(s) | Notes |
|--------------------------------------------|---------------------|-------|
| update annotate-snippets to latest version |                     |       |
| teach cargo to pass annotate-snippets flag | @estebank           |       |
| add ui test mode comparing new output      |                     |       |
| switch default nightly rustc output        |                     |       |

### Production use of annotate-snippets

| Task                                     | Owner(s) or team(s)  | Notes                          |
|------------------------------------------|----------------------|--------------------------------|
| switch default rustc output              |                      |                                |
| release notes                            |                      |                                |
| switch ui tests to only check new output |                      |                                |
| Dedicated reviewer                       | ![Team][] [compiler] | @estebank will be the reviewer |

### Standard reviews

| Task             | Owner(s) or team(s)  | Notes |
|------------------|----------------------|-------|
| Standard reviews | ![Team][] [compiler] |       |

### Top-level Rust blog post inviting feedback

| Task                                       | Owner(s) or team(s) | Notes |
|--------------------------------------------|---------------------|-------|
| Top-level Rust blog post inviting feedback |                     |       |
[cargo-lints]: https://github.com/rust-lang/cargo/issues/12235
