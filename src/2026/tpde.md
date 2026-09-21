# Add an experimental TPDE compilation option

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Point of contact | @ZuseZ4                                                                          |
| Status           | Proposed                                                                         |
| What and why     | Significant speedup of debug builds                                              |
| Roadmap          |                                                                                  |
| Other tracking issues |             |
| Zulip channel    | [gsoc/TPDE codegen backend for rustc][zulip] |

[zulip]: https://rust-lang.zulipchat.com/#narrow/channel/421156-gsoc/topic/Idea.3A.20TPDE.20codegen.20backend.20for.20rustc

## Summary

This project is a copy of the GSoC project which hadn't been selected by Google last time.


## Motivation
Compile times are the number one complain for a lot of users. We already have multiple ongoing projects to improve total compile times, e.g. via the parallel frontend, or Wild as a potential parallel linker. This project is orthogonal and focus purely on improving the compile times of debug builds, when using our LLVM backend.

### The status quo

LLVM is our default codegen backend, for both debug and release builds. LLVM is known to have very bad compile time performance in debug mode. TPDE is an experiment which accepts LLVM IR as input and tries to replace LLVM's debug compilation at a fraction of the compile time. TPDE can target x86-64 and AArch64, which covers our most popular targets. TPDE claims 10-20x faster compile times than LLVM O0. Earlier experiments using TPDE in Rust's LLVM codegen backend have shown very promising results, especially they were able to already compile larger and significant crates.

### What we propose to do about it

Distribute the TPDE library via rustup, and allow using it as a codegen backend for LLVM. The goal is to support a sufficiently large subset of crates and demonstrate a significant speedup of debug builds on those crates, without major changes to the TPDE library itself. If the improvements we've seen replicate on such a larger scale, we can start talking about a follow-up project, to see what it takes to move TPDE from an experimental option for the LLVM backend to a stable one.


### Work items over the next 6 month

#### Usable implementation in nightly, with limitations.
| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Add a build step to bootstrap to build and distribute TPDE on supported targets | @TechnoPorg | 2 weeks |
| Add a `-Z tpde` option which uses TPDE instead of LLVM for debug builds | @TechnoPorg | 6 weeks |
| Performance engineering - Measure improvements, analyze where we don't see improvements over LLVM's O0 | @TechnoPorg | 8 weeks |
| Work with TPDE maintainers to fix low-hanging fruits in TPDE if they enable support for a larger set of crates | @TechnoPorg | 8 weeks |
| Review the changes in the compiler and bootstrap | @ZuseZ4 | throughout the project |


## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Medium        | dedicated reviewer  (@ZuseZ4)           |
| [bootstrap]| Small         | backup reviewer                         |

## Funding

The duration of the project is 6 months.

- Month 1-2 (Usable implementation in nightly)
  - This stage could already provide benefits for nightly users, at the cost of frequent failures to compile code, or to achieve speedups.
  - Some of the work is already done, needs review.
- Month 3-6 (Usability and Performance improvements)
  - Performance optimizations
  - Fixing some of the biggest issues encountered by nightly users

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor - Nightly implementation | $6k + $1.2k overhead  | No | |
| Reviewer - Nightly implementation | covered | Full | |


## Frequently asked questions
Q: How does this compare to the Cranelift backend? 
A: TPDE has shown larger compile time improvements than Cranelift, and it operates on LLVM IR, so it can be added to our default LLVM backend and does not require a new codegen backend.
