# Getting Rust for Linux into stable Rust: language features

| Metadata         |                                                                                  |
|:-----------------|----------------------------------------------------------------------------------|
| Contact | @tomassedovic                                                                    |
| Status           | Proposed                                                                         |
| Tracking issue   | [rust-lang/goals#116]                                               |
| Zulip channel    | [#t-lang][channel-t-lang], [#rust-for-linux][channel-rust-for-linux]             |
| [lang] champion | @joshtriplett |
| [lang-docs] champion | @traviscross |

[channel-t-lang]: https://rust-lang.zulipchat.com/#narrow/channel/213817-t-lang
[channel-rust-for-linux]: https://rust-lang.zulipchat.com/#narrow/channel/425075-rust-for-linux

## Summary

Continue working towards Rust for Linux on stable. In particular, this goal is focused on the language features.

## Motivation

Getting the Linux kernel to build with stable Rust and, more generally, supporting the needs of the Linux kernel to make Rust a success there, has been a priority for the Rust project and a previous flagship goal: [2024H2](https://goals.rust-lang.org/2024h2/rfl_stable.html), [2025H1](https://goals.rust-lang.org/2025h1/rfl.html).

One of the key areas are language features, given the impact they could have on the kernel if they were to change, especially those that may require changes on potentially many source files and/or that may not be easy to workaround with conditional compilation.

Thus, this project goal focuses on continuing the work that has been done in the last year around those language features.

### The status quo

The Linux kernel, at the time of writing, relies on a few Rust language unstable features:

  - [`arbitrary_self_types`](https://github.com/rust-lang/rust/issues/44874).
  - [`derive_coerce_pointee`](https://github.com/rust-lang/rust/issues/123430).

In addition, there are others that we will likely want to start using in the future, such as:

  - [`asm_const_ptr`](https://github.com/rust-lang/rust/issues/128464).
  - Field projections: [project goal](https://github.com/rust-lang/goals/pull/329).
  - In-place initialization / Emplacement / ...: [project goal](https://github.com/rust-lang/goals/pull/344).

For completeness, on the library side, we use:

  - [`cfg(no_fp_fmt_parse)`](https://github.com/rust-lang/rust/pull/86048).
  - [`file_with_nul`](https://github.com/rust-lang/rust/issues/141727).

Furthermore, on the compiler side, we use:

  - `compiler_builtins`.
  - [`used_with_arg`](https://github.com/rust-lang/rust/issues/93798).

As well as a set of compiler flags and other features which have their own [project goal](https://github.com/rust-lang/goals/pull/346).

### The next 6 months

Ideally, the Linux kernel will not need to rely on any of the existing language unstable features, nor the related library and compiler features mentioned above, by the time the period is over. This would be a major milestone for both the Rust Project and Rust for Linux.

However, any progress on any feature (or an alternative to using a given unstable feature) would be welcome. In particular, finishing the work around `arbitrary_self_types` and `derive_coerce_pointee` and stabilizing them would be considered a major success.

### The "shiny future" we are working towards

Longer-term, the Linux kernel does not rely on any language-related unstable feature anymore, including possible future ones that the kernel may need to start using, such as the others listed above.

## Design axioms

An important design axiom that still applies from previous iterations is "**Don't let perfect be the enemy of good**". The primary goal is to offer stable support for the particular use cases that the Linux kernel requires. Wherever possible we aim to stabilize features completely, but if necessary, we can try to stabilize a subset of functionality that meets the kernel developers' needs while leaving other aspects unstable.

## Ownership and team asks

| Task                         | Owner(s) or team(s) | Notes                                      |
|------------------------------|---------------------|--------------------------------------------|
| Discussion and moral support | ![Team][] [lang]    | Continue the Rust for Linux <-> Rust calls |

Which features get finished and stabilized depends on bandwidth and other constraints on both the Rust and the Rust for Linux sides, but generally we expect they will follow the usual pattern as we have done before.

### Finish and stabilize `arbitrary_self_types` and `derive_coerce_pointee`

| Task                         | Owner(s) or team(s)   | Notes |
|------------------------------|-----------------------|-------|
| Discussion and moral support | ![Team][] [lang]      |       |
| Finalize remaining work      | @dingxiangfei2009     |       |
| Author Reference PR          | @dingxiangfei2009     |       |
| Review/revise Reference PR   | ![Team][] [lang-docs] |       |
| Author stabilization report  | @dingxiangfei2009     |       |
| Author stabilization PR      | @dingxiangfei2009     |       |
| Stabilization decision       | ![Team][] [lang]      |       |

## Frequently asked questions
