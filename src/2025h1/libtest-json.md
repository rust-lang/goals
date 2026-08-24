# Finish the libtest json output experiment

| Metadata           |                                    |
| :--                | :--                                |
| Point of contact   | @epage                             |
| Status             | Accepted                           |
| Zulip channel      | N/A                                |
| Tracking issue     | [rust-lang/goals#255] |

## Summary

Finish the [libtest json experiment](https://rust-lang.github.io/rfcs/3558-libtest-json.html).

## Motivation

[libtest](https://github.com/rust-lang/rust/tree/master/library/test)
is the test harness used by default for tests in cargo projects.
It provides the CLI that cargo calls into and enumerates and runs the tests discovered in that binary.
It ships with rustup and has the same compatibility guarantees as the standard library.

Before 1.70, anyone could pass `--format json` despite it being unstable.
When this was fixed to require nightly,
this helped show [how much people have come to rely on programmatic output](https://www.reddit.com/r/rust/comments/13xqhbm/announcing_rust_1700/jmji422/).

Cargo could also benefit from programmatic test output to improve user interactions, including
- [Wanting to run test binaries in parallel](https://github.com/rust-lang/cargo/issues/5609), like `cargo nextest`
- [Lack of summary across all binaries](https://github.com/rust-lang/cargo/issues/4324)
- [Noisy test output](https://github.com/rust-lang/cargo/issues/2832) (see also [#5089](https://github.com/rust-lang/cargo/issues/5089))
- [Confusing command-line interactions](https://github.com/rust-lang/cargo/issues/1983) (see also [#8903](https://github.com/rust-lang/cargo/issues/8903), [#10392](https://github.com/rust-lang/cargo/issues/10392))
- [Poor messaging when a filter doesn't match](https://github.com/rust-lang/cargo/issues/6151)
- [Smarter test execution order](https://github.com/rust-lang/cargo/issues/6266) (see also [#8685](https://github.com/rust-lang/cargo/issues/8685), [#10673](https://github.com/rust-lang/cargo/issues/10673))
- [JUnit output is incorrect when running multiple test binaries](https://github.com/rust-lang/rust/issues/85563)
- [Lack of failure when test binaries exit unexpectedly](https://github.com/rust-lang/rust/issues/87323)

Most of that involves shifting responsibilities from the test harness to the test runner which has the side effects of:
- Allowing more powerful experiments with custom test runners (e.g. [`cargo nextest`](https://crates.io/crates/cargo-nextest)) as they'll have more information to operate on
- Lowering the barrier for custom test harnesses (like [`libtest-mimic`](https://crates.io/crates/libtest-mimic)) as UI responsibilities are shifted to the test runner (`cargo test`)

### The status quo

### The next 6 months

1. Experiment with potential test harness features
2. Experiment with test reporting moving to Cargo
3. Putting forward a proposal for approval

### The "shiny future" we are working towards

- Reporting shifts from test harnesses to Cargo
- We run test harnesses in parallel

## Design axioms

- Low complexity for third-party test harnesses so its feasible to implement them
- Low compile-time overhead for third-party test harnesses so users are willing to take the compile-time hit to use them
- Format can meet expected future needs
  - Expected is determined by looking at what other test harnesses can do (e.g. fixture, paramertized tests)
- Format can evolve with unexpected needs
- Cargo perform all reporting for tests and benches

## Ownership and team asks

*This section defines the specific work items that are planned and who is expected to do them. It should also include what will be needed from Rust teams. The table below shows some common sets of asks and work, but feel free to adjust it as needed. Every row in the table should either correspond to something done by a contributor or something asked of a team. For items done by a contributor, list the contributor, or ![Heap wanted][] if you don't yet know who will do it. For things asked of teams, list ![Team][] and the name of the team. The things typically asked of teams are defined in the [Definitions](#definitions) section below.*

| Task                              | Owner(s) or team(s)       | Notes |
|-----------------------------------|---------------------------|-------|
| Discussion and moral support      | ![Team][] [testing-devex], [cargo], [libs]     |       |
| Prototype harness                 | @epage                    |       |
| Prototype Cargo reporting support | @epage                    |       |
| Write stabilization report        | @epage                    |       |

### Definitions

Definitions for terms used above:

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