# Promoting Parallel Front End

| Metadata                     |                                          |
| :--------------------------- | ---------------------------------------- |
| Contact             | @petrochenkov                            |
| Status                       | Accepted                                 |
| Tracking issue               | [rust-lang/goals#121]       |
| See also                     | [rust-lang/rust#113349]                  |
| Zulip channel                | [#t-compiler/wg-parallel-rustc][channel] |
| Roadmap                      | Fast Builds                              |
| [wg-parallel-rustc] champion | @petrochenkov                            |

## Summary

Continue with stabilization and performance improvements to the parallel front-end, continuing from the [2025h2 goal](https://goals.rust-lang.org/2025h2/parallel-front-end.html).

## Motivation

Verify and resolve the few remaining issues(especially related to incremental compilation), stabilize the feature, and try to further improve parallel compilation performance through various means.

### The status quo

The parallel front-end has progressed well over the past year. We resolved [deadlock] issues and several [ICEs][ICE], added the `rustc-rayon` dependency for the parallel front-end to the rustc working tree, and enabled the parallel front-end in bootstrap. Thanks to everyone for their efforts!

We recently fixed (in rust-lang/rust#151509) a major issue related to race conditions when incremental compilation executed in parallel.

The inconstancy of query cycle errors between parallel and serial compilers is another point we should pay attention to.

Furthermore, we do not yet have robust [testing] mechanisms to guarantee correctness and consistence of the  parallel front-end. we'll therefore dedicate focused effort toward on a new test suite for parallel front-end.

In addition, we need to document and enhance the support of surrounding tools for the parallel frontend, such as rustc-perf and Cargo.

After that, we will start the feature stabilization process to push parallel front end into the stable release channel.

Performance improvement would be long-term work. We are considering potential measures to speed up the parallel front-end, such as reducing data contention, adopting finer-grained parallelism, and enabling parallel name resolution and macro expansion.

### What we propose to do about it

- Resolve remaining issues in the issue [list][open issues].
- Build a parallel front end test suite to ensure the robustness and prevent regressions
- Enable the parallel front end in Cargo.
- Add more benchmarks for the parallel front end to rustc-perf
- Write a feature stabilization report and submit it for compiler FCP
- Continue improving parallel compilation performance, by parallelizing more parts of the compiler, like name resolution and macro expansion, and reducing data contention.

### Work items over the next year

| Task                         | Owner(s) or team(s)  | Notes |
| ---------------------------- | -------------------- | ----- |
| Issue Resolving              | @wg-parallel-rustc   |       |
| Building the test suit       | @ywxt                |       |
| Tools supporting             | @ywxt                |       |
| Stabilization report         | @petrochenkov        |       |
| Discussion and moral support | @wg-parallel-rustc   |       |

## Team asks

| Team       | Support level | Notes |
| ---------- | ------------- | ----- |
| [wg-parallel-rustc]    |     Large      |   Discussion and Implementation    |
| [compiler] |      Small         |  Code Reviews     |

## Frequently asked questions

[channel]: https://rust-lang.zulipchat.com/#narrow/channel/187679-t-compiler.2Fwg-parallel-rustc/
[ICE]: https://github.com/rust-lang/rust/issues?q=label%3AA-parallel-compiler%20label%3AI-ICE
[deadlock]: https://github.com/rust-lang/rust/issues?q=label%3AA-parallel-compiler+deadlock
[testing]: https://github.com/rust-lang/rust/issues/118698
[open issues]: https://github.com/rust-lang/rust/issues?q=state%3Aopen%20label%3AA-parallel-compiler
[big issue]: https://github.com/rust-lang/rust/issues/141540
