# Stabilize cargo-script

| Metadata         |                                                                                  |
|:-----------------|----------------------------------------------------------------------------------|
| Contact | @epage                                                                           |
| Status           | Accepted                                                                         |
| Tracking issue   | [rust-lang/goals#119]                                               |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |
## Summary

Stabilize support for "cargo script", the ability to have a single file that contains both Rust code and a `Cargo.toml`.

## Motivation

Being able to have a Cargo package in a single file can reduce friction in development and communication,
improving bug reports, educational material, prototyping, and development of small utilities.

### The status quo

Today, at minimum a Cargo package is at least two files (`Cargo.toml` and either `main.rs` or `lib.rs`).
The `Cargo.toml` has several required fields.

To share this in a bug report, people resort to
- Creating a repo and sharing it
- A shell script that cats out to multiple files
- Manually specifying each file
- Under-specifying the reproduction case (likely the most common due to being the easiest)

To create a utility, a developer will need to run `cargo new`, update the
`Cargo.toml` and `main.rs`, and decide on a strategy to run this (e.g. a shell
script in the path that calls `cargo run --manifest-path ...`).

### The next 6 months

The support is already implemented on nightly.
The goal is to stabilize support.
With [RFC #3502] and [RFC #3503] approved, the next steps are being tracked in [rust-lang/cargo#12207].

At a high-level, this is
- Add support to the compiler for the frontmatter syntax
- Add support in Cargo for scripts as a "source"
- Polish

### The "shiny future" we are working towards

## Design axioms

- In the trivial case, there should be no boilerplate.  The boilerplate should scale with the application's complexity.
- A script with a couple of dependencies should feel pleasant to develop without copy/pasting or scaffolding generators.
- We don't need to support everything that exists today because we have multi-file packages.

## Ownership and team asks

Tracking issue [cargo#12207](https://github.com/rust-lang/cargo/issues/12207):

| Task                         | Owner(s) or team(s) | Notes |
|------------------------------|---------------------|-------|
| Discussion and moral support | ![Team][] [cargo], [compiler]   |       |
| Ensure Cargo implementation  | @epage             |       |

### Implement language feature `frontmatter`

Tracking issue [#136889](https://github.com/rust-lang/rust/issues/136889):

| Task                              | Owner(s) or team(s)                | Notes |
|-----------------------------------|------------------------------------|-------|
| Rustc implementation              | @epage |       |
| Rust-analyzer implementation      | @epage |       |
| Standard reviews                  | ![Team][] [compiler]               |       |
| Author call for testing blog post | @epage |       |

### Stabilize language feature `frontmatter`

| Task                        | Owner(s) or team(s)   | Notes         |
|-----------------------------|-----------------------|---------------|
| Author Reference PR         | @epage                |               |
| Review/revise Reference PR  | ![Team][] [lang-docs] | @ehuss        |
| Author stabilization report | @epage                |               |
| Author stabilization PR     | @epage                |               |
| Stabilization decision      | ![Team][] [lang]      |               |

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
