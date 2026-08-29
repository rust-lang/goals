# Run more tests for GCC backend in the Rust's CI

| Metadata              |                                         |
| --------------------- | --------------------------------------- |
| Contact      | @GuillaumeGomez                         |
| Status                | Proposed                                |
| Other tracking issues | [rust-lang/compiler-team#891]           |
| Zulip channel         | [#rustc-codegen-gcc][rustc-codegen-gcc] |
| [compiler] champion   | @WesleyWiser                            |
| [infra] champion      | @marcoieni                              |
| Tracking issue        | [rust-lang/goals#402]      |


[rustc-codegen-gcc]: https://rust-lang.zulipchat.com/#narrow/channel/386786-rustc-codegen-gcc
[rust-lang/compiler-team#891]: https://github.com/rust-lang/compiler-team/issues/891

## Summary

Over the next six months, we will do the following:

 * Run GCC backend's test with GCC backend built sysroot
 * Add new ui tests annotations (`//@ ignore-backend` and `//@ needs-backend`)
 * Run tests with GCC backend:
   * libcore tests
   * UI tests
   * [GCC backend specific tests][extra tests]
 * Add documentation in dev guide about how to run tests with the GCC backend

To be noted: this goal is only for linux x86_64 target.

## Motivation

Currently, most of the time spent on work on the GCC backend is to keep up to date with Rust's changes: we need to figure out which changes broke things on our side. And with a higher number of changes, this makes this process a lot longer. Running more GCC backend tests directly in Rust CI would allow to detect regressions directly when a change is made, making it much simpler to figure out what went wrong.

### The status quo

We want to make the GCC backend a first-class option for Rust developers. Many things remain to be done for us to reach this point, and reducing the time spent on syncs would allow us to focus on adding the missing parts and improve generated code.

### Adding new ui tests annotations

The new ui tests annotations (`//@ ignore-backend` and `//@ needs-backend`) will allow to run directly the ui tests without all the current filtering processes we put in place in the `rustc_codegen_gcc` repository. They are needed because a lot of tests are simply not possible to pass since they're tied to LLVM (like generated assembly code). It will also make it easier for Rust's developers to check if their changes work with GCC.

As a reminder, this goal will only target linux x86_64.

### Running more tests

Currently, only a few tests are run, and if they fail, they are not blocking a PR from being merged. The goal is to make them mandatory to succeed and also to run more tests:
  * libcore
  * UI tests
  * [GCC backend specific tests][extra tests]

### Adding documentation

It is important for rust developers to be able to test how changes impact the GCC backend. To do so, documentation is a must have and will be added as part of this goal.

## Ownership and team asks

> *This section lists out the work to be done and the asks from Rust teams. Every row in the table should either correspond to something done by a contributor or something asked of a team.*
>
> *For most goals, a single table will suffice, but you can also add subsections with `###`. We give several example subsections below that also demonstrate the most common kinds of goals. Remember that the items in the table only corresponds to what you plan to do over the next 6 months.*
>
> *For items done by a contributor, list the contributor, or ![Heap wanted][] if you don't yet know who will do it. The owner is ideally identified as a github username like `@ghost`.*
>
> *For items asked of teams, list ![Team][] and the name of the team, e.g. `![Team][] [compiler]` or `![Team][] [compiler], [lang]` (note the trailing `[]` in `![Team][]`, that is needed for markdown to parse correctly). For team asks, the "task" must be one of the tasks defined in [rust-project-goals.toml](../rust-project-goals.toml) or `cargo rpg check` will error.*

| Task                           | Owner(s) or team(s)           | Notes                                                                                 |
| ------------------------------ | ----------------------------- | ------------------------------------------------------------------------------------- |
| Adding new ui annotations      | @GuillaumeGomez               |                                                                                       |
| Running more GCC backend tests | @GuillaumeGomez               |                                                                                       |
| Standard reviews               | ![Team][] [infra], [compiler] | with the help of t-infra and t-compiler for reviews and ensuring is done as they want |

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
[extra tests]: https://github.com/rust-lang/rustc_codegen_gcc/tree/master/tests/run
