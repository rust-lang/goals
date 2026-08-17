# Publish first version of `rustc_public` on crates.io

| Metadata         |                                   |
| :--------------- | --------------------------------- |
| Point of contact | @makai410                         |
| Status           | Proposed                          |
| [project-rustc-public] champion | @celinval          |
| Tracking issue   | [rust-lang/goals#266]             |
| Timespan         | 2026-2027                         |
| Zulip channel    | [#project-rustc-public][channel]  |

[channel]: https://rust-lang.zulipchat.com/#narrow/channel/320896-project-rustc-public

## Summary

Publish `rustc_public` crate(s) to crates.io to allow tool developers to create applications on top of rustc, and extract code information from compiled Rust crates and their dependencies without using compiler internal APIs.

## Motivation

Accessing rustc internals via `#![feature(rustc_private)]` exposes comprehensive intermediate state, albeit cumbersome to maintain for third-party tool developers.

The notable painful parts of developing tools based on `rustc_private` are:
- Frequent breakage of `rustc_private` APIs without notice.
- The need to reverse-engineer complex compiler internals.
- The requirement to use a nightly toolchain.
- No ability to consume rustc APIs from crates.io with Cargo.

The fundamental problem is that rustc is built to enable rapid compiler development and is shaped around its own invariants and performance needs, but not for external ergonomics — outside consumers, on the other hand, would be perfectly happy with something more friendly and permissive at the cost of some overhead and precision.

For what it's worth, there was a good talk describing the situation well at RustWeek 2026: [Out of tree access to compiler state (Alona Enraght-Moony at RustWeek)][talk].

[talk]: https://youtu.be/ExxxtADP-t8?si=_RNkvn5kAh_wmkkw

### The status quo

Over the past few years, we have been working on the `rustc_public` (formerly `stable_mir`) effort, introducing a SemVer-compliant API to rustc around the needs of tool developers. As of today it allows them to analyze and extract information from compiled Rust crates without directly depending on rustc internals.

Nonetheless, today it is consumed not unlike any other internal compiler crate.
It doesn't have any explicit version, is not available on crates.io, and must be imported via an `extern crate` statement.
In other words, multi-version rustc support, plus semantic versioning, are still hypothetical for now.

By publishing it on crates.io, we will fulfill our promise to provide SemVer APIs, with notice and guidance whenever a breaking change is necessary.

### What we propose to do about it

1. Bring the infrastructure in the rust-lang/rustc_public repository to a point where it is *good enough* for publishing `rustc_public` on crates.io. *Good enough* here means:
    - being able to detect any breaking changes that might occur due to compiler updates.
    - having build automation that handles the compilation and test execution.
    - having extensible testing infrastructure.
    - achieving over 70% line coverage across the `rustc_public` crate.

2. Provide *comprehensive* documentation for both contributors and users. *Comprehensive* here means:
    - rustc developers have clear guidelines for updates.
    - future contributors know how to properly expose a new API in `rustc_public`.
    - users know how to develop tools with rustc_public through the documentation covering several common scenarios.

3. Design and implement an MVP of the `rustc_public` driver ([rust-lang/rustc_public#69](https://github.com/rust-lang/rustc_public/issues/69)).
    - We will use this crate, with its much smaller interface, as a pilot to establish how crates from the rustc_public repository are published and maintained, laying the groundwork for publishing `rustc_public` itself.

At the end, we will publish the first version of `rustc_public` on crates.io.

#### Design axioms
- Enable tool developers to implement sophisticated analyses with low maintenance cost.
- Do not compromise the development and innovation speed of rustc.
- Crates should follow semantic versioning.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Reframe the [MCP](https://github.com/rust-lang/compiler-team/issues/949) | @celinval ||
| Configure and test Josh sync CI | @makai410 ||
| Integrate SemVer checks | @makai410 ||
| Implement build and test automation | @makai410 ||
| Establish the testing infrastructure | @makai410 ||
| Write documentation | @makai410 ||
| Design and implement an MVP of the driver | @makai410 ||
| Publish the crate | @celinval ||

## Team asks
| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Small         | Discussion and moral support            |
| [project-rustc-public] | Medium        | Standard reviews            |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | Ask | No  |            |


## Frequently asked questions

### Would it impose additional maintenance responsibility on rustc developers when `rustc_public` is published to crates.io?

No, and we won't impose the SemVer compatibility maintenance burden on rustc maintainers. We want to reduce the current maintenance burden
on rustc maintainers.
Ultimately we can remove the `rustc_public` crate from rustc, and this goal is hopefully a step toward getting everything ready for that to happen. 
