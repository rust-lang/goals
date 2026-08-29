# Production-ready cranelift backend

| Metadata            |                                    |
| :------------------ | :--------------------------------- |
| Contact    | @folkertdev                        |
| Status              | Proposed                           |
| Flagship            | Flexible, fast(er) compilation     |
| Tracking issue      | [rust-lang/goals#397] |
| Zulip channel       |                                    |
| [compiler] champion | @bjorn3                            |

## Summary

This project aims to get the rust cranelift codegen backend to a production-ready state. Specifically, with this work completed, we'd be confident to recommend it as the default for local development, e.g. with `cargo test` or `cargo run`.

## Motivation

Compile-time performance consistently comes up as a limiting factor in the use and enjoyment of rust. Through extensive tracking and the hard work of T-perf and others, performance has gradually improved over the years. However, in practical terms, a 1% improvement here and there is not sufficient to truly improve users' workflows.

The cranelift codegen backend (`rustc_codegen_cranelift`) is one of the most concrete ways we have to deliver a serious improvement to compile times in the typical development cycle.  In our measurements on larger projects (e.g. Zed, Tauri, hickory-dns), we currently see roughly a 20% reduction of code generation time, translating to around 5% speedup of total compilation time for clean builds.

Currently, this backend still has some serious limitations that real-world code bases run into. This project goal aims to get the `rustc_codegen_cranelift` to a state where we can confidently recommend it for local development.

## Status quo

The cranelift backend is already very capable, and well-integrated into the rust project. It is able to re-use a lot of infrastructure and code, for instance via `rustc_codegen_ssa`. It can be enabled on a per-project basis in the `Cargo.toml`:

```toml
# This line needs to come before anything else in Cargo.toml
cargo-features = ["codegen-backend"]

[profile.dev]
codegen-backend = "cranelift"
```

It is however not quite stable and reliable enough for real use. Most large projects run into one or two missing features, meaning that `rustc_codegen_cranelift` cannot be used. Furthermore, small changes in the project and/or its dependencies could run into limitations of `rustc_codegen_cranelift`, so while experimentation with the backend is highly encouraged, we cannot currently recommend it for serious local development.

## The next 6 months

We plan to work on fixing the following cranelift limitations:

- supporting unwinding (at least on Linux and macOS)
- fixing remaining ABI issues
- a better approach for supporting (SIMD) intrinsics

As a side-effect we will also look at improving code sharing with rustc, for instance by moving more logic into `rustc_codegen_ssa`. These changes reduce the maintenance burden of the other backends (cranelift, but also `rustc_codegen_gcc`), and generally improves the code which is still quite over-fit to LLVM

With these items completed, we believe that we can confidently recommend the use of the cranelift backend for local development and CI builds on linux and macOS.

### The "shiny future" we are working towards

We believe we can get to a production-ready `rustc_codegen_cranelift` implementation for Linux and macOS, on `x86_64` and `aarch64`, within the context of this project goal. The problems and solutions are mapped out, and there are many similarities between these targets.

Ultimately, we'd like to provide the same level of quality for Windows, but this is much more challenging. Progress on this front is highly dependent on additional funding, and will take considerably more time.

While this project aims to deliver a production-ready backend, it is by no means feature-complete at that point. One major missing feature is debug info (i.e. debugger support).

More long-term we see the cranelift backend as an extremely promising approach for decreasing compile times, and substantially improving developer experience. However, significant technical work remains to be done to achieve that goal. We believe that the best next step towards this goal is to first get cranelift ready for widespread use, and then collectively work on further improving performance.

## Ownership and team asks

| Task                 | Owner(s) or team(s)                 | Notes                                                                                                                                |
| -------------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Acquire funding      | [Trifecta Tech Foundation]          | We will only be able to work on this project if it is funded                                                                         |
| Standard reviews     | ![Team][] [compiler]                | Casual improvements to `rustc_codegen_ssa` and other parts of the backend                                                            |
| Dedicated reviewer   | ![Team][] [compiler]                | Larger changes to `rustc_codegen_ssa`. While not strictly required, we think having a dedicated reviewer will speed up our progress. |
| Deploy to production | ![Team][] [wg-compiler-performance] | If possible, track and show `rustc_codegen_cranelift` performance. See note below for more details.                                  |
| Do the work          | @bjorn3, @folkertdev                |                                                                                                                                      |

Note: the `wg-compiler-performance` ask is a nice to have, and it is clear to both the goal owners and the working group that:
- it depends on [another goal][rustc-perf infra] for the technical ability to exist in the first place, with its own requirements and timeline
- and when it's possible to handle `rustc_codegen_cranelift` in `rustc-perf`, deployment and availability could *also* have practical requirements like funding
- and that `wg-compiler-performance` can accept the ask conditionally on the above external requirements it doesn't control

[Trifecta Tech Foundation]: https://github.com/trifectatechfoundation
[rustc-perf infra]: https://goals.rust-lang.org/2025h2/rustc-perf-improvements.html

## Frequently asked questions
