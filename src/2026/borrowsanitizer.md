# BorrowSanitizer

| Metadata            |                                    |
| :--                 | :--                                |
| Contact    | @icmccorm                          |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#624] |
| Zulip channel       | N/A                                |
| [compiler] champion | @RalfJung                          |
| [opsem] champion    | @RalfJung                          |
| [lang] champion     | @tmandry                           |


## Summary

We are building BorrowSanitizer: an LLVM-based instrumentation tool for finding violations of Rust's aliasing model. In 2026, we want to make it feature-complete and useful in practice.

## Motivation

### The status quo

Developers rely on Miri to validate their programs against Rust's latest [Tree Borrows](https://dl.acm.org/doi/10.1145/3735592) aliasing model. However, Miri cannot find these Rust-specific aliasing bugs when they are caused by foreign function calls. Miri's performance is also several orders of magnitude slower than native execution. As Rust is increasingly being in security-critical C and C++ applications, like [Android](https://source.android.com/docs/setup/build/rust/building-rust-modules/overview) and [Chromium](https://chromium.googlesource.com/chromium/src.git/+/HEAD/docs/rust.md), developers will need a method for finding aliasing errors that can scale across language boundaries.

### What we propose to do about it

We are developing BorrowSanitizer to fix this tooling gap. Much like AddressSanitizer, MemorySanitizer, and other LLVM-based tools, BorrowSanitizer inserts
checks during compilation to detect errors at run-time. Its purpose is to find violations of Rust's newest [Tree Borrows](https://dl.acm.org/doi/10.1145/3735592)
aliasing model, as well as accesses out-of-bounds and use-after-free errors.

BorrowSanitizer relies on changes to the Rust compiler, an LLVM instrumentation pass, and a runtime library. We modified the compiler to emit special "retag" intrinsics that indicate when
references are created and updated. Our LLVM pass lowers these intrinsics into runtime calls that associate each pointer with "provenance" metadata (see [provenance RFC #3559](https://rust-lang.github.io/rfcs/3559-rust-has-provenance.html)). We validate provenance before memory accesses to detect undefined behavior.

Our primary goal is for BorrowSanitizer to be useful in practice. This will require broad support for Rust, C, and C++ language features. We want to achieve better performance than Miri while fully supporting the different features of Tree Borrows.

### Work items over the next year
Throughout 2026, we will complete the remaining features needed for BorrowSanitizer to have parity with Miri for detecting aliasing violations. We will finish contributing the retag intrinsics described in [our previous project goal](https://goals.rust-lang.org/2025h2/codegen_retags.html) and evaluate when and how the rest of BorrowSanitizer should be integrated with the compiler.

One topic for discussion is whether the BorrowSanitizer itself should live in a subtree of rust-lang/rust, with a new Github repo under rust-lang, or as an independent project.

| Task                                            | Owner(s)      | Notes                          |
| ----------------------------------------------- | ------------- | ------------------------------ |
| Complete [MCP](https://github.com/rust-lang/compiler-team/issues/958) and implementation for retag intrinsics       | @icmccorm     |                                |
| Feature-parity with Miri                        | @icmccorm, BorSan Team |  Garbage collection, error reporting, atomics, interception, and more!  |
| Automated Evaluation                      | @icmccorm, BorSan Team   |  Self-hosted [crater](https://github.com/rust-lang/crater) instance, benchmarking.                             |
| Full compiler integration       | @icmccorm     | Have the runtime, instrumentation pass, and other components available upstream in nightly.                              |

BorrowSanitizer is open source and available on [GitHub](https://github.com/borrowsanitizer/bsan), and we welcome any contributions. We will post monthly status updates on [our website](https://borrowsanitizer.com/) throughout 2026. We are available at any point for Q/A on [Zulip](https://bsan.zulipchat.com/). @icmccorm will be the primary contact for the [BorrowSanitizer Team](https://borrowsanitizer.com/about.html#team).

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] |    Medium     | Champion: @RalfJung. Design discussions, PR review, and upstream integration. |
| [opsem]    |    Medium     | Champion: @RalfJung.                                |
| [lang]     |    Small      | Champion: @tmandry. General support and guidance.   |
| [infra]    |    Small      | Upstream integration.                               |

## Frequently asked questions
