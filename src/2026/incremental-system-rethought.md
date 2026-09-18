# Incremental Systems Rethought

| Metadata            |                                    |
| :--                 | :--                                |
| Contact    | @blyxyas                           |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#641] |
| Roadmap             | Fast Builds                        |
| [compiler] champion | @jackh726                          |
| Zulip channel       | [#t-compiler/incremental-systems-rethought][zulip] |

[zulip]: https://rust-lang.zulipchat.com/#narrow/channel/635945-t-compiler.2Fincremental-systems-rethought

## Summary

Design, propose, and implement a redesign of the incremental system to allow for shared common bases between different Rustc invocations with different arguments (i.e. `cargo build` and `cargo check`)

There's already an [on-going RFC], based on [this blog post and talk] by the same author (and this project goal's contact)
As my other project goals, this is a formalization of an already existing effort.

## Motivation

The incremental system (i.e. the system that takes care of reusing the dep-graph and keeping track of changes on
compiler invocations) does not take into account inter-activity communication. So, `cargo build` and `cargo check`,
while they share a lot of common meaning and thus should share much in a perfect incremental system, in reality
trigger a recompilation if performed one after the other.

This project goal will design, propose and implement the necessary changes for `check`, `build` and possibly some
3rd-parties such as `clippy` to share a common incremental ground when performed in a compatible way.

So, when a user runs `cargo check` and then `cargo build`, `cargo build` would reuse much of the work done by `cargo check`.
`cargo check` and `cargo clippy` (if implemented) would also reuse a lot of work. `cargo build` and `cargo test` could also
reuse some e.g. parsing information

### The status quo

Quoting from the earlier mentioned RFC:

The current model for incremental recompilations doesn't share progress between compiler activities, leading to unnecessary
rebuilds. Users notice redundant compilations, as "Changes in workspaces trigger unnecessary rebuilds" was submitted as
[a big complaint in the compiler performance][perf-survey].

---

This affects all users that use a typical `cargo check` -> `cargo build` -> `cargo test` workflow. Even on codebases
that don't use e.g. `cfg(test)` or similar profile-based `cfg` attributes, it will rebuild the workspace 3 times.

Summing to the already slow compilation times, having to rebuild the workspace on every different command is a pain
point discovered in the performance survey.

### The next 6 months

Over the next 6 months the plan is to address all review comments and concerns on [aforementioned RFC][on-going RFC] and
via working with the compiler team, merge an unstable initial implementation on the compiler. This implementation would
introduce a working base system where a `cargo build` -> `cargo check` workflow does not recompile.

After those 6 months, we'll focus on less systematic improvements. We'll work on starting the stabilization process
and hopefully get it close to the finish line. In the stabilization process, we'll try to make the API
as expandable as possible to allow for granularity in the recompilation tracking.

| Task            | Owner(s)           | Notes |
|-----------------|--------------------|-------|
| Design, propose | @blyxyas @jackh726 |       |
| Implementation  | @blyxyas           |       |

### The "shiny future" we are working towards

The end goal is to allow for smoother work-reutilization between compiler invocations with little to no overhead. With dynamic dependencies declaring which compiler flags are sensitive for the current codebase recompiling, and keeping an accurate track of macro invocations and a real and specific sense of which functions from dependencies the current crate depends on.

The shiny future, and the end goal of the whole roadmap is to avoid the following scenarios:

- Full recompilations on `cargo check` -> `cargo build` on all scenarios. (Some will need it due to e.g. build scripts, but the majority don't)
- Recompilations at all on `cargo build` -> `cargo check`
- Recompilations due to changes in unused functions from dependencies.
- Recompilations due to CLI flags that don't affect the current invocation (e.g. `-Clto` on `cargo check`)

The first step is completing the RFC and implementing it in an unstable stage.

## Team asks

| Team       | Support level | Notes |
|------------|---------------|-------|
| [compiler] | Medium        |       |

## Frequently asked questions

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | $75,000-$120,000 | Partial | [AWS](https://aws.amazon.com/) |

Greater amount of funding would allow for more time spent working on the goal. If the funding goal is not met, part-time work will be given instead.

## Target timeline

The duration of the project is 12 months. Starting from the agreed start date ("Month 1"), the target timeline is:

- Month 1-3:
  - Resolve all concerns, merge RFC. (In parallel with experimentation)
  - Stablish formal communication with the notable teams. (i.e. cargo and compiler)
  - Experiment with `cargo build` building off of `cargo check`.
- Month 3-6:
  - Declare semi-stable fingerprinting format following any known schema.
  - Notion of data dependencies & atomic levels in the compiler.
  - Have some kind of `cargo build` <-> `cargo check` compatibility merged in the compiler.
- Month 6-12:
  - Implement data dependency diffing algorithm in the compiler for the different stages.
  - Polishing, perf measurements, integration testing...
  - Measure interaction with official rustc-driver projects, such as Clippy or Miri.
  - (secondary) Measure interaction with unofficial rustc-driver projects.
- Month 12-future:
  - Allow for stable interoperability with other tools.
  - (Eventual) Stabilization.

[on-going RFC]: https://github.com/rust-lang/rfcs/pull/3881/
[this blog post and talk]: https://blog.goose.love/posts/improving-the-incremental-system-in-the-rust-compiler/
[perf-survey]: https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results/#incremental-rebuilds
