# Stabilize the next-generation trait solver

| Metadata              |                                           |
|:----------------------|-------------------------------------------|
| Contact      | @lcnr                                     |
| Status                | Accepted                                  |
| What and why          | Replace the existing trait solver with a sound, maintainable implementation that unblocks soundness fixes and async features |
| Roadmap               | Project Zero                 |
| Roadmap               | Just add async                            |
| Tracking issue        | [rust-lang/goals#113]        |
| Other tracking issues | [rust-lang/rust#107374]                   |
| Zulip channel         | [#t-types/trait-system-refactor][channel] |
| Highlight             | Next-generation trait solver              |
| [types] champion      | @lcnr                                     |
| [lang] champion       | @nikomatsakis                             |
| Funding contact       | [Hexcat](https://hexcat.nl/) |

[channel]: https://rust-lang.zulipchat.com/#narrow/channel/364551-t-types.2Ftrait-system-refactor

## Summary

Stabilize `-Znext-solver=globally`, replacing the existing trait solver implementation entirely.

## Motivation

The next-generation trait solver is intended to fully replace the existing type system components responsible for proving trait bounds, normalizing associated types, and much more. This should fix many long-standing (soundness) bugs, enable future type system improvements, and improve compile-times.

### The status quo

There are multiple type system unsoundnesses blocked on the next-generation trait solver: [project board][unsoundnesses]. Desirable features such as coinductive trait semantics and perfect derive, where-bounds on binders, and better handling of higher-ranked bounds and types are also stalled due to shortcomings of the existing implementation.

[unsoundnesses]: https://github.com/orgs/rust-lang/projects/44/views/1

Since starting to work on this at the EOY 2022, we've:
- stabilized its use in coherence checking in Rust 1.84
- replaced the use of `chalk` in `rust-analyzer`
- done full crater runs and all large regressions are either fixed or intended
- improved performance and diagnostics, even if they is still more work to do

### What we propose to do about it

Stabilize `-Znext-solver=globally` and rip out the old implementation
- finish triaging the crater and and resolve remaining issues tracked in the [project board](https://github.com/orgs/rust-lang/projects/61/views/1)
- achieve performance parity with the old solver
- write an RFC for the cycle semantics used by the new trait solver
- document the behavior of the new trait solver and write stabilization reports

### Work items over the next year

| Task | Owner(s) | Notes |
| -----| -------- | ----- |
| figure out and resolve remaining blockers |@lcnr | |
| achieve performance parity | @lcnr | |
| cycle semantics RFC | @lcnr, @nikomatsakis | |
| complete stabilization report | @lcnr | |
| stabilization | @lcnr | |
| remove old implementation | @lcnr | |

## Team asks

| Team    | Support level | Notes                                       |
| ------- | ------------- | ------------------------------------------- |
| [lang]  | Medium        | Stabilization decision for user facing changes |
| [types] | Large         | Stabilization decision, ongoing review work |

## Funding

| Purpose           | Cost | Funded | Sponsor(s)     |
|-------------------|------|--------|----------------|
| Core contribution | Ask | Full   | Google, Anon1, Anon2 |
| Acceleration      | Ask | No     |

## Frequently asked questions

### What do I do with this space?
