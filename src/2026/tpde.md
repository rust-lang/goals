# Add an experimental TPDE compilation option

| Metadata              |                                              |
| :-------------------- | -------------------------------------------- |
| Contact               | @ZuseZ4                                      |
| Status                | Proposed                                     |
| What and why          | Significant speedup of debug builds          |
| Roadmap               | Fast Builds                                  |
| Other tracking issues |                                              |
| Zulip channel         | [gsoc/TPDE codegen backend for rustc][zulip] |
| [compiler] champion   | @ZuseZ4                                      |

[zulip]: https://rust-lang.zulipchat.com/#narrow/channel/421156-gsoc/topic/Idea.3A.20TPDE.20codegen.20backend.20for.20rustc

## Summary

Add an experimental TPDE-based compilation mode to rustc's LLVM backend to substantially reduce debug build times. This project is based on a [proposal for GSoC 2026][gsoc], which was not selected for the program.

[gsoc]: https://github.com/rust-lang/google-summer-of-code/blob/3d1a0ed860a204e19f966d23efd135d2aa16864d/README.md#tpde-codegen-backend-for-rustc

## Motivation

Compile times are the number one complain for a lot of users. We already have multiple ongoing projects to improve total compile times, e.g. via the parallel frontend, or Wild as a potential parallel linker. This project is orthogonal and focus purely on improving the compile times of debug builds, when using our existing LLVM backend.

### The status quo

LLVM is our default codegen backend, for both debug and release builds. LLVM is known to have very bad compile time performance in debug mode. 

TPDE-llvm is an experiment which accepts LLVM IR as input and tries to replace LLVM's debug compilation at a fraction of the compile time. TPDE-llvm can target x86-64 and AArch64 elf, which covers our most popular targets. TPDE claims 10-20x faster compile times than LLVM O0. Earlier experiments using TPDE-llvm in Rust's LLVM codegen backend have shown very promising results, especially they were able to already compile larger and significant crates.

### What we propose to do about it

Distribute the TPDE-llvm library via rustup, and allow using it as a codegen option of the existing LLVM backend. The goal is to support a sufficiently large subset of crates and demonstrate a significant speedup of debug builds on those crates, without major changes to the TPDE library itself. If the improvements we've seen replicate on such a larger scale, we can start talking about a follow-up project, to see what it takes to move TPDE-llvm from an experimental option for the LLVM backend to a stable one.


### Our shiny future
TPDE-LLVM has shown to be bug-free, and became an official sub-project of LLVM. Lessons learned were applied to major other non-elf targets. The debug compile times of backend dominated builds improved in the order of 50%. We can mix-and-match between LLVM O0, LLVM O3, and TPDE-llvm, depending on the needs of users.


### Work items over the next 6 month

#### Usable implementation in nightly, with limitations.
| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Add a build step to bootstrap to build and distribute `tpde-plugin.so` on supported targets | @TechnoPorg | 2 weeks |
| Add a `-Z tpde` option which uses TPDE-llvm instead of LLVM for debug builds | @TechnoPorg | 3 weeks |
| Work with TPDE maintainers to fix low-hanging fruits in TPDE if they enable support for a larger set of crates | @TechnoPorg | 8 weeks |
| Set our new backend up to run Gankra's [abi-cafe]. Report bugs to TPDE maintainers | @TechnoPorg | 1 week |
| Performance engineering - Measure improvements, analyze where we don't see improvements over LLVM's O0 | @TechnoPorg | 8 weeks |
| Review the changes in the compiler and bootstrap | @ZuseZ4 | throughout the project |

[abi-cafe]: https://github.com/Gankra/abi-cafe/tree/main

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Medium        | dedicated reviewer  (@ZuseZ4)           |
| [bootstrap]| Small         | backup reviewer (@Kobzol) |

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
A: Cranelift requires maintaining it's own backend, whereas TPDE-llvm is a drop-in replacement for our LLVM backend. TPDE-llvm therefore reuses the existing LLVM codegen backend. Compile time improvements from TPDE-llvm seem to show significant additional compile time improvements over Cranelift

Q: How do you handle features that are covered by LLVM, but not by TPDE-llvm?  
A: We expect to keep LLVM O0 (or later maybe cg\_clif) as a fallback for such cases. We expect TPDE-llvm to handle almost all cases, and as such the compile-time cost of calling back to LLVM should be neglible. We explicitely do not want to split our existing LLVM codegen backend to adjust IR generation depending on LLVM O0 vs TPDE-llvm usage.

Q: How do you handle ABI bugs?  
A: TPDE-LLVM should implement LLVM's undocumented ABI or report a function/call as unsupported. A mismatch is a [LLVM] bug. (TPDE author in [discussion].)

[discussion]: https://rust-lang.zulipchat.com/#narrow/channel/546987-goals.2Fproposed/topic/goals.23791.3A.20Propose.20a.20project.20goal.20for.20a.20TPDE.20backend/near/626241139
