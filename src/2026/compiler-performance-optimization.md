# Compiler performance optimizations

| Metadata        |                               |
| :-------------- | ----------------------------- |
| Contact         | @nnethercote                  |
| Funding contact | [Hexcat](https://hexcat.nl/)  |
| Status          | Proposed                      |
| Roadmap         | Fast Builds                   |
| Timespan        | 2026-2027                     |

## Summary

Make `rustc` faster through sustained, incremental, profile-driven optimization.
This work will improve performance in the new trait solver and borrow checker (Polonius)
and revisit optimization opportunities across the compiler with improved tooling.

## Motivation

There are two basic ways to speed up the Rust compiler.

- Big improvements. Large projects like *pipelined compilation* and *the
  parallel backend* have given large speed-ups, e.g. 10-50% across a wide range
  of benchmarks. However, such projects can be difficult to complete. For
  example, the parallel front-end was begun in 2018 and is still in progress,
  having gone through multiple rounds of stasis and reanimation.

- Small improvements. This is steady, incremental, profile-driven work. Each
  improvement typically results in single-digit percentage improvements, often
  on smaller ranges of benchmarks, as described in the long-running "How to
  speed up the Rust compiler" blog series. It is less glamorous work but it
  adds up over time and it historically has been the primary driver of speed,
  particularly over the period 2016-2023 when compiler speed increased by
  roughly 3x. It has slowed down in the past couple of years, however, due to
  diminishing returns.

The big improvements are still worth pursuing, but the small improvements path
has been re-energized by recent developments.

- The [new trait solver](./next-solver.md) and
  [new borrow checker (Polonius)](./polonius.md) are close to shipping.
  Both are multi-year projects that make Rust more expressive and capable. Both
  currently cause significant compile-time regressions and require effort to
  get their performance acceptable. Small improvements will be the solution
  here, as was the case when the current borrow checker (NLL) was introduced in
  2018.

- New tooling, in the form of LLMs, has greatly improved. The recent huge
  increases in security vulnerability discovery are well known across many
  projects. We have seen evidence that LLMs are similarly helpful for
  performance work; an expert human using an LLM can analyze profiles and find
  and implement optimizations better than an expert human can alone. Previously
  tapped-out seams of optimization will likely reopen all across the compiler.
  This LLM usage aligns with the Rust project's values because the analysis
  provides all the value; LLM code generation is not required.

This work requires experience with a variety of profiling tools and
optimization techniques.

### Work items over the next year

| Task | Owner(s) | Notes |
| ---- | -------- | ----- |
| Profile the compiler and find optimization opportunities | @nnethercote | |
| Improve performance of the new trait solver | @nnethercote | |
| Improve performance of the new borrow checker | @nnethercote | |
| Improve performance across the compiler | @nnethercote | |

## Team asks

| Team | Support level | Notes |
| ---- | ------------- | ----- |
| [compiler] | Small | Reviews for targeted optimizations |
| [types] | Small | Reviews for trait solver optimizations |

## Funding

Funding will support @nnethercote's profiling, benchmarking, and optimization
work. Additionally, extra funding can support 1-2 additional engineers to form
a small team. Contact [Hexcat](https://hexcat.nl/) to fund this goal.

| Purpose | Cost | Funded | Sponsor(s) |
| ------- | ---- | ------ | ---------- |
| Compiler performance optimization work | TBD | No | |

## Frequently asked questions
