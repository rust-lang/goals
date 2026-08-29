# Delegation

| Metadata              |                         |
| :-------------------- | :---------------------- |
| Contact      | @petrochenkov           |
| Status                | Not accepted            |
| Zulip channel         | N/A                     |
| Other tracking issues | [rust-lang/rust#118212] |
| [compiler] champion   | @petrochenkov           |

## Summary

Finish implementation of the delegation language as described in [RFC 3530], and update the RFC
with more detailed design based on the implementation experience.

## Motivation

This proposal falls under the [Efficient code reuse](https://github.com/rust-lang/rfcs/issues/349)
umbrella.

Rust doesn't have the sort of data inheritance common for object oriented languages, in which
derived data structure can inherit from some base data structure and automatically use its methods
that way.
In Rust this pattern is typically expressed through composition, in which the "base" data structure
is put into the "derived" data structure as a (possibly nested) field or some similar kind of
sub-object.
Newtypes (`struct Derived(Base);`) are an especially popular example of such pattern.

With composition methods that in other languages could be potentially inherited automatically need
to be implemented manually (possibly with help of macros).
Such trivial implementations may create a lot of boilerplate, and even prevent people from using
newtypes when it would be appropriate for type safety.

This proposal aims to support a sugar that would allow to avoid such boilerplate in cases
similar to inheritance and newtypes, and in other cases too if they fit into the same basic
mechanism, while staying in limited syntactic budget.

### The status quo

The base parts of the RFC, and some advanced features were implemented in 2024, reported bugs were
fixed and the feature is already in a usable state, even if the functionality is limited.

The detailed checklist for the sub-features can be found on the
[tracking issue](https://github.com/rust-lang/rust/issues/118212).

### The "shiny future" we are working towards

Rust programmers will be able to conveniently delegate implementations of functions (struct methods
in particular) to other already implemented functions (for some fields of those structs in particular).

Some special support for newtypes will be available, like adjustment of the return type from the
wrapped type to newtype.

Some higher level sugar, like delegating the whole trait implementation at once will also be
supported.

### The next 6 months

We plan to complete implementation of the sub-features as described in the
[tracking issue](https://github.com/rust-lang/rust/issues/118212) and update the RFC with the more
detailed design based on the implementation experience.

## Design axioms

Our design of delegation feature is guided by a number of principles described in the [RFC 3530].

1. Limit the syntactic budget for the feature, so a delegated function looks more like an import,
  than a function implementation.
2. Use statistics from existing code to select functionality that we should fit into that limited
  budget.
3. Support more rarely used functionality if it fits into the same implementation scheme and
doesn't extend the syntactic budget.

## Ownership and team asks

**Owner:** @petrochenkov

| Task                         | Owner(s) or team(s)       | Notes      |
| ---------------------------- | ------------------------- | ---------- |
| Discussion and moral support | ![Team][] [lang]          |            |
| Author RFC                   | @petrochenkov             | [RFC 3530] |
| Implementation               | @bryanskiy, @petrochenkov |            |
| Standard reviews             | ![Team][] [compiler]      |            |
| Design meeting               | ![Team][] [lang]          |            |
| RFC decision                 | ![Team][] [lang]          |            |

[RFC 3530]: https://github.com/rust-lang/rfcs/pull/3530
[tracking issue]: https://github.com/rust-lang/rust/issues/118212

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

TBD
