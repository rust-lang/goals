# Stabilize Rust for Linux compiler features

| Metadata            |                                    |
|:--------------------|------------------------------------|
| Contact    | @tomassedovic                      |
| Status              | Accepted                           |
| What and why        | Stabilize compiler features the Linux kernel depends on so it can build with only stable Rust |
| Timespan            | 2026-2027                          |
| Roadmap             | Rust for Linux                     |
| Tracking issue      | [rust-lang/goals#407] |
| Zulip channel       | [#rust-for-linux][channel-rfl]     |
| [compiler] champion | @WesleyWiser                       |

[channel-t-compiler]: https://rust-lang.zulipchat.com/#narrow/channel/131828-t-compiler
[channel-rfl]: https://rust-lang.zulipchat.com/#narrow/channel/425075-rust-for-linux

## Summary

Develop and stabilize compiler features that Rust for Linux uses. This is a continuation of the existing Rust for Linux effort.

## Motivation

Getting the Linux kernel to build with stable Rust and, more generally, supporting the needs of the Linux kernel to make Rust a success there, has been a priority for the Rust project and a previous flagship goal: [2024H2](https://goals.rust-lang.org/2024h2/rfl_stable.html), [2025H1](https://goals.rust-lang.org/2025h1/rfl.html).

One of the key areas are compiler features, which encompass a wide range of topics: architecture/target-related flags, sanitizers, mitigations, performance/optimization-oriented flags, and so on.

The primary goal is to offer stable support for the particular use cases that the Linux kernel requires. We're sticking to the **Don't let perfect be the enemy of good** approach.  Wherever possible we aim to stabilize features completely, but if necessary, we can try to stabilize a subset of functionality that meets the kernel developers' needs while leaving other aspects unstable.


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
  - [`-Zsanitizer=kernel-hwaddress` and `-Zsanitizer-recover=kernel-hwaddress`](https://github.com/rust-lang/compiler-team/issues/975) (arm64).
  - [`-Zsanitize-kcfi-arity`](https://github.com/rust-lang/rust/issues/138311) (x86_64).

There is also the [`build-std` project goal](https://github.com/rust-lang/rfcs/pull/3873) support that we need as well (or, rather, only "`build-core`" for the Linux kernel), and the [sanitizers project goal](https://goals.rust-lang.org/2025h2/stabilization-of-sanitizer-support.html).

### What we propose to do about it

We track each compiler feature we're interested in and move them forward towards stabilization. Historically, this was done by either compiler developers or the Rust for Linux team members.

### Work items over the next year

Ideally, the Linux kernel would not need to rely on any of the existing compiler unstable features by the time the period is over, but that is fairly unlikely. Thus, any progress on any feature (or an alternative to using a given unstable feature) would be welcome.

In particular, finishing the work and stabilizing the features that the kernel is already using upstream would be considered a major success.

Longer-term, the Linux kernel does not rely on any compiler-related unstable feature anymore, except for those that may need to be added in the future for different reasons, such as:

  - New hardware features.
  - New mitigations.
  - New sanitizers.
  - New architectures.

For that reason, this goal is conceptually never ending, even if we may reach points where no unstable compiler feature is actually used.

## Team asks

| Team       | Support level | Notes                 |
|------------|---------------|-----------------------|
| [compiler] | Medium        | Reviews, RfL meetings |
