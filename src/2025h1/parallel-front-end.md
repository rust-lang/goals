# Promoting Parallel Front End

| Metadata         |                                          |
|:-----------------|------------------------------------------|
| Contact | @SparrowLii                              |
| Status           | Accepted                                 |
| Tracking issue   | [rust-lang/goals#121]       |
| Zulip channel    | [#t-compiler/wg-parallel-rustc][channel] |
[channel]: https://rust-lang.zulipchat.com/#narrow/channel/187679-t-compiler.2Fwg-parallel-rustc/
## Summary

Continue to parallelize front-end stabilization and performance improvements, continuing from the [2024h2 goal](https://goals.rust-lang.org/2024h2/parallel-front-end.html).

## Motivation

There are still some occasional deadlock issues, and in environments with high thread counts (>16) performance may be reduced due to data races.

### The status quo

Many current [issues] reflect ICE or deadlock problems that occur during the use of parallel front end. We need to resolve these issues to enhance its robustness. We also need theoretical algorithms to detect potential deadlocks in query systems.

The current parallel front end still has room for further improvement in compilation performance, such as parallelization of HIR lowering and macro expansion, and reduction of data contention under more threads (>= 16).

We can use parallel front end in bootstrap to alleviate the problem of slow build of the whole project.

Cargo does not provide an option to enable the use of parallel front end, so it can only be enabled by passing rustc options manually.

### The next 6 months

- Solve reproducible deadlock issues via [tests](https://github.com/rust-lang/rust/pull/132051).
- Enable parallel frontends in bootstrap.
- Continue to improve parallel compilation performance, with the average speed increase from 20% to 25% under 8 cores and 8 threads.
- Communicate with Cargo team on the solution and plan to support parallel front end.

### The "shiny future" we are working towards

We need a detection algorithm to theoretically prove that the current query system and query execution process do not bring potential deadlock problems.

The existing rayon library implementation is difficult to fundamentally eliminate the deadlock problem, so we may need a better scheduling design to eliminate deadlock without affecting performance.

The current compilation process with `GlobalContext` as the core of data storage is not very friendly to parallel front end. Maybe try to reduce the granularity (such as modules) to reduce data competition under more threads and improve performance.

## Design axioms

The parallel front end should be:
- safe: Ensure the safe and correct execution of the compilation process
- consistent: The compilation result should be consistent with that in single thread
- maintainable: The implementation should be easy to maintain and extend, and not cause confusion to developers who are not familiar with it.

[da]: ../about/design_axioms.md

## Ownership and team asks

**Owner:** @SparrowLii and Parallel Rustc WG own this goal

| Task                         | Owner(s) or team(s)  | Notes |
| ---------------------------- | -------------------- | ----- |
| Implementation               | @SparrowLii          |       |
| Author tests                 | @SparrowLii          |       |
| Discussion and moral support | ![Team][] [compiler] |       |

## Frequently asked questions
[ICE]: https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3AWG-compiler-parallel+ice
[deadlock]: https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3AWG-compiler-parallel+deadlock
[test]: https://github.com/rust-lang/rust/issues/118698
[issues]: https://github.com/rust-lang/rust/labels/WG-compiler-parallel
