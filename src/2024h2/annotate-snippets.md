# Use annotate-snippets for rustc diagnostic output

| Metadata         |                                    |
|------------------|------------------------------------|
| Contact | @Muscraft                          |
| Status           | Accepted                           |
| Tracking issue   | [rust-lang/goals#123] |
| Zulip channel    | N/A                                |
## Summary

Switch to annotate-snippets for rendering rustc's output, with no loss of functionality or visual regressions.

## Motivation

Cargo has been [adding its own linting system][cargo-lints], where it has been using annotate-snippets to try and match Rust's output. This has led to duplicate code between the two, increasing the overall maintenance load. Having one renderer that produces Rust-like diagnostics will make it so there is a consistent style between Rust and Cargo, as well as any other tools with similar requirements like miri, and should lower the overall maintenance burden by rallying behind a single unified solution.

### The status quo

Currently rustc has its own Emitter that encodes the theming properties of compiler diagnostics. It has handle all of the intricancies of terminal support (optional color, terminal width querying and adapting of output), layout (span and label rendering logic), and the presentation of different levels of information. Any tool that wants to approximate rustc's output for their own purposes, needs to use a third-party tool that diverges from rustc's output, like annotate-snippets or miette. Any improvements or bugfixes contributed to those libraries are not propagated back to rustc. Because the emitter is part of the rustc codebase, the barrier to entry for new contributors is artificially kept high than it otherwise would be.

annotate-snippets is already part of the rustc codebase, but it is disabled by default, doesn't have extensive testing and there's no way of enabling this output through cargo, which limits how many users can actually make use of it.

### The next 6 months

- annotate-snippets rendered output reaches full partity (modulo reasonable non-significant divergences) with rustc's output
- rustc is fully using annotate-snippets for their output.

### The "shiny future" we are working towards

The outputs of rustc and cargo are fully using annotate-snippets, with no regressions to the rendered output. annotate-snippets grows its feature set, like support for more advanced rendering formats or displaying diagnostics with more than ASCII-art, independently of the compiler development cycle.

## Design axioms

*This section is optional, but including [design axioms][da] can help you signal how you intend to balance constraints and tradeoffs (e.g., "prefer ease of use over performance" or vice versa). Teams should review the axioms and make sure they agree. [Read more about design axioms][da].*
- **Match rustc's output**: The output of annotate-snipepts should match rustc, modulo reasonable non-significant divergences
- **Works for Cargo (and other tools)**: annotate-snippets is meant to be used by any project that would like "Rust-style" output, so it should be designed to work with any project, not just rustc.
[da]: https://goals.rust-lang.org/about/design_axioms.html

## Ownership and team asks

**Owner:** @estebank, @Muscraft

*Identify a specific person or small group of people if possible, else the group that will provide the owner*

This section defines the specific work items that are planned and who is expected to do them. It should also include what will be needed from Rust teams.

* Subgoal:
    * Describe the work to be done and use `↳` to mark "subitems".
* Owner(s) or team(s):
    * List the owner for this item (who will do the work) or ![Help wanted][] if an owner is needed.
    * If the item is a "team ask" (i.e., approve an RFC), put ![Team][] and the team name(s).
* Status:
    * List ![Help wanted][] if there is an owner but they need support, for example funding.
    * Other needs (e.g., complete, in FCP, etc) are also fine.

*Adjust the table below; some common examples are shown below.*

| Task                                       | Owner(s) or team(s)  | Notes |
|--------------------------------------------|----------------------|-------|
| Standard reviews                           | ![Team][] [compiler] |       |
| Top-level Rust blog post inviting feedback |                      |       |

### Reach output parity for rustc/annotation-snippets

| Task                                          | Owner(s) or team(s) | Notes |
|-----------------------------------------------|---------------------|-------|
| Port a subset of rustc's UI tests             | @Muscraft           |       |
| Make list of current unnaddressed divergences | @Muscraft           |       |
| address divergences                           | @Muscraft           |       |

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

## Frequently asked questions

### What do I do with this space?

*This is a good place to elaborate on your reasoning above -- for example, why did you put the design axioms in the order that you did? It's also a good place to put the answers to any questions that come up during discussion. The expectation is that this FAQ section will grow as the goal is discussed and eventually should contain a complete summary of the points raised along the way.*

[cargo-lints]: https://github.com/rust-lang/cargo/issues/12235
