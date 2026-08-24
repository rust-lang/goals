# Normative Documentation for Sound `unsafe` Rust

| Metadata         |                                                                                                                |
| :--              | :--                                                                                                            |
| Point of contact | @PLeVasseur                                                                                                    |
| Status           | Accepted                                                                                                       |
| What and why     | Normative documentation for common `unsafe` patterns so safety-critical developers have authoritative guidance |
| Roadmap          | Safety-Critical Rust                                                                                           |
| Tracking issue   | [rust-lang/goals#644]                                                                             |
| Zulip channel    | N/A                                                                                                            |
| [opsem] champion | @RalfJung                                                                                                      |


## Summary

The Safety-Critical Rust Consortium will investigate real-world safety-critical codebases to identify common `unsafe` patterns, then work with t-opsem to create normative documentation in the Rust Reference and standard library docs.

## Motivation

### The status quo

Rust's `unsafe` documentation has gaps. The Rustonomicon self-describes as "incomplete" and the Unsafe Code Guidelines Reference is "largely abandoned." Neither is normative. When developers need to know whether a specific `unsafe` pattern is sound, they often find no authoritative answer.

This matters acutely in safety-critical domains. Automotive, medical, and aerospace software must demonstrate code behaves correctly. Without normative documentation, developers cannot build rigorous safety cases for `unsafe` code.

The [zerocopy crate](https://github.com/google/zerocopy) established a model: annotate every `unsafe` block with rationale citing official Rust documentation; when documentation is insufficient, work with t-opsem to improve it (e.g., [rust-lang/rust#114902] for `addr_of!` semantics).

### What we'll do

We'll apply the zerocopy model systematically, starting with [Eclipse iceoryx2](https://github.com/eclipse-iceoryx/iceoryx2), a zero-copy IPC framework with ~3,300 `unsafe` usages. The process:

1. Catalog `unsafe` patterns in the codebase
2. Identify which patterns lack normative documentation
3. Work with t-opsem to establish correct safety contracts
4. Submit PRs to the Rust Reference and std docs

Priority areas include cross-process synchronization, memory-mapped regions, cross-process atomics, and `UnsafeCell` in shared memory—patterns common in systems programming but underdocumented.

### The "shiny future"

Developers writing `unsafe` code can cite authoritative documentation for safety contracts. Safety-critical projects can build rigorous safety cases. The patterns documented benefit the entire ecosystem, not just safety-critical users.

## Team asks

| Team        | Support level | Notes                                                                   |
|-------------|---------------|-------------------------------------------------------------------------|
| [opsem]     | Large         | Review unsafe patterns, establish safety contracts, guide documentation |
| [libs]      | Small         | PR reviews for core/std public documentation; feedback on approach.     |
| [lang]      | Small         | Feedback on language semantics questions as needed                      |
| [lang-docs] | Small         | Standard PR reviews for Rust Reference                                  |

## FAQ

### Why iceoryx2?

It's a pure-Rust safety-critical IPC framework with extensive `unsafe` usage (shared memory, lock-free structures, FFI). The patterns it uses appear throughout the systems programming ecosystem.

### Won't this overwhelm t-opsem?

No. Most usages fall into ~20 pattern categories, of which 5-7 need new normative documentation. We're asking t-opsem to help with genuinely novel questions, not review thousands of individual usages.

### How does this differ from cataloging undefined behavior?

We document **safety contracts** (what you must do to be sound), not undefined behavior (what happens when you're unsound). The Rust Reference's UB list is intentionally non-exhaustive; we respect that approach.
