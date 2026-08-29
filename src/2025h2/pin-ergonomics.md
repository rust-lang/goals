# Continue Experimentation with Pin Ergonomics

| Metadata            |                                                                                  |
| :--                 | :--                                                                              |
| Contact    | @frank-king                                                                      |
| Status              | Proposed                                                                         |
| Flagship            | Beyond the `&`                                                                   |
| Tracking issue      | [rust-lang/goals#389]                                               |
| Zulip channel       | N/A (an existing stream can be re-used or new streams can be created on request) |
| [compiler] champion | @oli-obk                                                                         |
| [lang] champion     | @traviscross                                                                     |


## Summary

Continue experimenting with and fleshing out the design and semantics for the pin ergonomics experiment.

## Motivation

Several flagship Rust features, such as `async` and generators, depend on the ability to ensure data will not move in memory.
The way Rust achieves this is through the `Pin` wrapper, but pinning has notoriously poor ergonomics.
Exploring how the ergonomics can be improved by integrating pinning into the language better could help unblock advancements in other Rust features.

### The status quo

Pinning exists but few people like it.
We have an experiment for improving the ergonomics around pinning and some initial PRs have landed, but we would like to build more sustained momentum on it.

### The next 6 months

- introduce `&pin mut|const place` borrowing syntax (parsing #135731 ready to merge, lowering and  borrowck not started yet)
- pattern matching of `&pin mut|const T` types (#139751 under review)
- introduce `&pin pat` pattern syntax
- support `&pin mut|const T` -> `&|&mut T` coercion (requires `T: Unpin` of `&pin mut T` -> `&mut T`)
- support auto borrowing of `&pin mut|const place` in method calls with `&pin mut|const self` receivers

### The "shiny future" we are working towards
## Design axioms

## Ownership and team asks

| Task                              | Owner(s) or team(s)  | Notes |
| --------------------------------- | -------------------- | ----- |
| Implementation                    | @frank-king          |       |
| Standard reviews                  | ![Team][] [compiler] |       |
| Design meeting                    | ![Team][] [lang]     |       |
| Author call for testing blog post | @frank-king          |       |

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

### What do I do with this space?
