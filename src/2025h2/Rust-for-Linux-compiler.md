# Getting Rust for Linux into stable Rust: compiler features

| Metadata            |                                                                              |
| :--                 | :--                                                                          |
| Contact    | @tomassedovic                                                                |
| Status              | Proposed                                                                     |
| Tracking issue      | [rust-lang/goals#407]                                           |
| Zulip channel       | [#t-compiler][channel-t-compiler], [#rust-for-linux][channel-rust-for-linux] |
| [compiler] champion | @WesleyWiser                                                                 |

[channel-t-compiler]: https://rust-lang.zulipchat.com/#narrow/channel/131828-t-compiler
[channel-rust-for-linux]: https://rust-lang.zulipchat.com/#narrow/channel/425075-rust-for-linux

## Summary

Continue working towards Rust for Linux on stable. In particular, this goal is focused on the compiler features.

## Motivation

Getting the Linux kernel to build with stable Rust and, more generally, supporting the needs of the Linux kernel to make Rust a success there, has been a priority for the Rust project and a previous flagship goal: [2024H2](https://goals.rust-lang.org/2024h2/rfl_stable.html), [2025H1](https://goals.rust-lang.org/2025h1/rfl.html).

One of the key areas are compiler features, which encompass a wide range of topics: architecture/target-related flags, sanitizers, mitigations, performance/optimization-oriented flags, and so on.

Thus, this project goal focuses on continuing the work that has been done in the last year around compiler features.

### The status quo

The Linux kernel, at the time of writing, relies on some Rust compiler unstable features such as:

  - [`-Zbranch-protection`](https://github.com/rust-lang/rust/issues/113369) (arm64).
  - [`-Zcf-protection`](https://github.com/rust-lang/rust/issues/93754) (x86_64).
  - [`-Zcrate-attr`](https://github.com/rust-lang/rust/issues/138287).
  - [`-Zdebuginfo-compression`](https://github.com/rust-lang/rust/issues/120953).
  - [`-Zdirect-access-external-data`](https://github.com/rust-lang/rust/issues/127488) (loongarch).
  - `-Zfixed-x18` (arm64).
  - [`-Zfunction-return`](https://github.com/rust-lang/rust/issues/116853) (x86).
  - `-Zfunction-sections`.
  - [`-Zno-jump-tables`](https://github.com/rust-lang/rust/issues/116592) (x86_64).
  - [`-Zunpretty=expanded`](https://github.com/rust-lang/rust/issues/43364).
  - [`-Zsanitizer=kernel-address`](https://github.com/rust-lang/rust/issues/123615) (arm64, riscv64).
  - [`-Zsanitizer=shadow-call-stack`](https://github.com/rust-lang/rust/issues/123615) (arm64, riscv64).
  - [`-Zsanitizer=kcfi` and `-Zsanitizer-cfi-normalize-integers`](https://github.com/rust-lang/rust/issues/123479) (arm64, riscv64, x86_64).

There are others that we will want to start using in the future, such as:

  - [`-Zharden-sls`](https://github.com/rust-lang/rust/issues/116851) (x86_64).
  - [`-Zindirect-branch-cs-prefix`](https://github.com/rust-lang/rust/pull/140740) (x86).
  - [`-Zmin-function-alignment`](https://github.com/rust-lang/rust/issues/82232).
  - [`-Zrandomize-layout`](https://github.com/rust-lang/rust/issues/106764).
  - [`-Zregparm`](https://github.com/rust-lang/rust/issues/131749) (x86_32).
  - [`-Zreg-struct-return`](https://github.com/rust-lang/rust/issues/116973) (x86_32).
  - [`-Zretpoline` and `-Zretpoline-external-thunk`](https://github.com/rust-lang/rust/pull/135927) (x86).
  - [`-Zsanitizer=kernel-hwaddress` and `-Zsanitizer-recover=kernel-hwaddress`](https://github.com/rust-lang/rust/issues/123615) (arm64).
  - [`-Zsanitize-kcfi-arity`](https://github.com/rust-lang/rust/issues/138311) (x86_64).

Relatedly, there is also the [`build-std` project goal](https://github.com/rust-lang/goals/pull/331) support that we need as well (or, rather, only "`build-core`" for the Linux kernel), and the [sanitizers project goal](https://github.com/rust-lang/goals/pull/337).

### The next 6 months

Ideally, the Linux kernel would not need to rely on any of the existing compiler unstable features by the time the period is over, but that is fairly unlikely. Thus, any progress on any feature (or an alternative to using a given unstable feature) would be welcome.

In particular, finishing the work and stabilizing the features that the kernel is already using upstream would be considered a major success.

### The "shiny future" we are working towards

Longer-term, the Linux kernel does not rely on any compiler-related unstable feature anymore, except for those that may need to be added in the future for different reasons, such as:

  - New hardware features.
  - New mitigations.
  - New sanitizers.
  - New architectures.

For that reason, this goal is conceptually never ending, even if we may reach points where no unstable compiler feature is actually used.

## Design axioms

An important design axiom that still applies from previous iterations is "**Don't let perfect be the enemy of good**". The primary goal is to offer stable support for the particular use cases that the Linux kernel requires. Wherever possible we aim to stabilize features completely, but if necessary, we can try to stabilize a subset of functionality that meets the kernel developers' needs while leaving other aspects unstable.

## Ownership and team asks

| Task                         | Owner(s) or team(s)  | Notes                                      |
| ---------------------------- | -------------------- | ------------------------------------------ |
| Discussion and moral support | ![Team][] [compiler] | Continue the Rust for Linux <-> Rust calls |

Which flags get finished and stabilized depends on bandwidth and other constraints on both the Rust and the Rust for Linux sides, but generally we expect they will follow the usual pattern as we have done before.

### Finish and stabilize a given `-Z...` flag

| Task                         | Owner(s) or team(s)     | Notes |
| ---------------------------- | ----------------------- | ----- |
| Discussion and moral support | ![Team][] [compiler]    |       |
| Finalize remaining work      | (depending on the flag) |       |
| Author stabilization report  | (depending on the flag) |       |
| Author stabilization PR      | (depending on the flag) |       |
| Stabilization decision       | ![Team][] [compiler]    |       |

## Frequently asked questions
