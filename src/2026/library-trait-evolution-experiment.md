# Library Trait Evolution Experiment


| Metadata         |                                                                      |
| :--              | :--                                                                  |
| Contact | @lcnr                                                                |
| What and why     | Enable library authors to more easily change their trait definitions |
| Status           | Proposed                                                             |
| Tracking issue   | [rust-lang/goals#712]                                   |
| Zulip channel    | N/A                                                                  |
| [lang] champion  | @tmandry                                                             |
| [types] champion | @lcnr                                                                |
| Funding contact  | [Hexcat](https://hexcat.nl/)                                         |


## Summary

Setup a Library Trait Evolution experiment. It works to unify the design of language features to make trait definitions less rigid. This includes [refined trait implementations](https://github.com/rust-lang/rfcs/pull/3245) and [Supertrait Auto-impl](https://github.com/rust-lang/rfcs/pull/3851), but also more targeted features for the standard library, such as [`#[rustc_must_implement_one_of]`](https://github.com/rust-lang/rust/issues/107460).

We plan to collect different use-cases and requirements here, both from the standard library and the wider ecosystem. We then use this to design, implement, and stabilize features solving these issues.

## Motivation

### The status quo

Trait evolution and trait hierarchy refactoring is a long-time pain point for Rust crates and especially the standard library. As crates evolve and grow, needs to restructure trait hierarchy to accommodate richer functionality often arises, both in the standard library and the broader Rust ecosystem.

This will be even more of an issue once we add new *implicit default auto trait bounds* to support [Immobile types and guaranteed destructors](https://goals.rust-lang.org/2026/move-trait.html) or the [Sized Hierarchy](https://goals.rust-lang.org/2026/scalable-vectors.html). For example in `trait Trait { fn foo<T>(); }` the generic parameter `T` will get an implicit `T: Move` bound whose removal is a breaking change. This makes immobile types a lot less useable with existing traits.

### Work items over the next year


| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Setup a Library Trait Evolution experiment and create its repository | @lcnr |
| Collect and explore different use-cases in a single repository | @lcnr  |
| Collaborate on the design and implementation of language features to solve these issues | @lcnr |


## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [lang]     | Medium        | Probably want to have a lang meeting at some point |
| [libs]     | Small         | We would like to test some of the desired standard library changes and get input on the design |
| [types]    | Medium        | Review and discuss type system changes if required                                         |

## Funding

| Purpose           | Cost | Funded | Sponsor(s)     |
|-------------------|------|--------|----------------|
| Contributor | Ask | Partial  | |

## Frequently asked questions
