# Stabilize rustdoc `doc_cfg` feature

| Metadata              |                                    |
| :--                   | :--                                |
| Contact      | @GuillaumeGomez                    |
| Status                | Proposed                           |
| Other tracking issues | [rust-lang/rust#43781]             |
| Zulip channel         | [#t-rustdoc][t-rustdoc]            |
| [rustdoc] champion    | @GuillaumeGomez                    |
| Tracking issue        | [rust-lang/goals#404] |

[t-rustdoc]: https://rust-lang.zulipchat.com/#narrow/channel/266220-t-rustdoc

## Summary

Over the next six months, we will do the following:

 * Finish the review and implementation of the feature.
 * Get users feedback.
 * Stabilize the feature.

## Motivation

This goal represents the final step for the stabilization of a long awaited, allowing to greatly improve documentation quality by providing information about under which conditions an item is available.

### The status quo

This has been one of the most asked rustdoc features recently. An [RFC](https://github.com/rust-lang/rfcs/pull/3631) was written and merged. We're now reaching the final steps before making this feature stable.

Without this feature, it's not possible to know when an item is available, you need to go read the source code to find out if there are `cfg` attributes or not. It will improve the situation for both doc readers and doc writers:
 1. For readers, they won't need to read source code anymore to find out when an item is available.
 2. For doc writers, they won't need to write down explicitly when an item is available or rely on nightly features.

### The next 6 months

> *Sketch out the specific things you are trying to achieve in this goal period. This should be short and high-level -- we don't want to see the design!*

### Finish implementation

A [pull request](https://github.com/rust-lang/rust/pull/138907) is open, maybe a few more review rounds are required but it should be close to maturity.

### Getting feedback

The next step will be to gather feedback, and in particular ensure that there is no bug. Here are the steps:

 1. Communication about the feature.
 2. Enable it by default on docs.rs

### Stabilization

If no bug is reported for enough time, the last step will be to stabilize the feature.

## Ownership and team asks

> *This section lists out the work to be done and the asks from Rust teams. Every row in the table should either correspond to something done by a contributor or something asked of a team.*
>
> *For most goals, a single table will suffice, but you can also add subsections with `###`. We give several example subsections below that also demonstrate the most common kinds of goals. Remember that the items in the table only corresponds to what you plan to do over the next 6 months.*
>
> *For items done by a contributor, list the contributor, or ![Heap wanted][] if you don't yet know who will do it. The owner is ideally identified as a github username like `@ghost`.*
>
> *For items asked of teams, list ![Team][] and the name of the team, e.g. `![Team][] [compiler]` or `![Team][] [compiler], [lang]` (note the trailing `[]` in `![Team][]`, that is needed for markdown to parse correctly). For team asks, the "task" must be one of the tasks defined in [rust-project-goals.toml](../rust-project-goals.toml) or `cargo rpg check` will error.*

| Task                          | Owner(s) or team(s) | Notes |
|-------------------------------|---------------------|-------|
| Standard reviews              | ![Team][] [rustdoc] | Merge [#138907] |
| Communicate about the feature | @GuillaumeGomez     |       |
| Enable it by default on docs.rs | @GuillaumeGomez   |       |
| Send pull request to stabilize the feature | @GuillaumeGomez     |       |

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
