# Implement Supertrait `auto impl`

| Metadata              |                                                                                                          |
| :--                   | :--                                                                                                      |
| Point of contact      | @dingxiangfei2009                                                                                        |
| Status                | Accepted                                                                                                 |
| What and why          | Automatically implement supertraits when a subtrait is implemented, enabling trait hierarchy refactoring |
| Roadmap               | Beyond the `&`                                                                                           |
| Roadmap               | Rust for Linux                                                                                           |
| [lang] champion       | @cramertj                                                                                                |
| Tracking issue        | [rust-lang/goals#636]                                                                       |
| Other tracking issues | [rust-lang/rust#149556]                                                                                  |
| Zulip channel         | N/A                                                                                                      |


## Summary

Within the 2026 goal period we strive for completion of the following items.
- Implementation of the core language features stipulated by the [RFC #3851].
- Continuous update on the RFC for errata to reflect necessary changes as implementation moves along.
- Resolve the `impl` overlapping question, possibly as an optional feature behind an associated feature gate.
- Field trial of the standard library trait refactoring with the `supertrait_auto_impl` feature gate.
- Implementation of the optional features and lints spelled in the RFC as much as possible.

## Motivation

Trait evolution and trait hierarchy refactoring is a long-time pain point for Rust crates and especially the standard library. As crates evolve and grow, needs to restructure trait hierarchy to accommodate richer functionality often arises, both in the standard library and the broader Rust ecosystem.

Supertrait `auto impl` has been viewed as a potential solution and a promising language feature to the trait evolution problem, such that we can avoid major and elaborate rewrites in downstream crates. In essence, the feature would avoid rewrites when associate items are moved into a new supertrait, which is a common scenario that this problem concerns about.

```rust
// Before refactoring

pub trait BigTrait {
    fn method_lower_level();
    fn method_higher_level();
}

// After refactoring

pub trait BigTrait: Supertrait {
    // This signals to the compiler that `Supertrait` implementation shall
    // be automatically derived with the right items, so that ...
    auto impl Supertrait;
    fn method_higher_level();
}

pub trait Supertrait {
    fn method_lower_level();
}

// ... this `impl BigTrait` continues to compile
impl BigTrait for MyType {
    fn method_lower_level() { .. }
    // because this method is resolved to `Supertrait::method_higher_level`
    // and used to derive the `impl Supertrait for MyType` automatically.
    fn method_higher_level() { .. }
}
```

### The status quo

Supertrait `auto impl` targets library authors who needs to refactor traits or design a trait hierarchy. A recurring topic arising from this activity is that with refined trait granularity, or smaller traits in other words, also comes many required `impl`s. The problem exacerbates when it is an upstream trait receiving a refactor. While this is rightfully a breaking change, downstream crates would also have to move trait items into new `impl` blocks as the original trait is broken into smaller supertraits. This major rewrite is often undesirable for library authors as this would discourage downstream users to upgrade the library. For standard library, this often means that the changes can only land on an Edition boundary.

### The next 6 months

| Task                                   | Owner(s)          | Notes |
|----------------------------------------|-------------------|-------|
| Implementation of the language feature | @dingxiangfei2009 |       |
| ...                                    |                   |       |

### The "shiny future" we are working towards

We would like to establish a mechanism in the language to automatically derive the required supertrait implementation using the items available in the subtrait implementation. The first step is to enable the resolution of the supertrait associated items in the subtrait implementation block to the corresponding supertraits when there is no risk of ambiguity.

`auto impl` blocks follows so that the default supertrait implementation could be supplied from the supertrait definition. We will also propose a feature to allow downstream trait users with explicit opt-out of the default supertrait implementation in case of overlapping implementation or need for customisation.

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    |               |                                         |
| [compiler] |               |                                         |
| [infra]    |               |                                         |
| [lang]     |  Medium | Team aligned already on the shape of the feature |
| [libs]     |               |                                         |
| [opsem]    |               |                                         |
| [types]    | Small | `r? types` when touching the type system. Expect that anything beyond "simple" types changes may be rejected or de-prioritized. [^types-small] |

[^types-small]: [See discussion.](https://rust-lang.zulipchat.com/#narrow/channel/144729-t-types/topic/2026.20Project.20Goals/near/573917638)

## Frequently asked questions
