# Bring the Async Rust experience closer to parity with sync Rust

| Metadata         |                                    |
|:-----------------|------------------------------------|
| Short title      | Async                              |
| Point of contact | @tmandry                           |
| Status           | Accepted                           |
| Flagship         | Yes                                |
| Tracking issue   | [rust-lang/goals#105] |
| Zulip channel    | [#wg-async][channel]               |
[channel]: https://rust-lang.zulipchat.com/#narrow/channel/187312-wg-async/
## Summary

Over the next six months, we will continue bringing Async Rust up to par with "sync Rust" by doing the following:

* Telling a complete story for the use of async fn in traits, unblocking wide ecosystem adoption,
* Improving the ergonomics of `Pin`, which is frequently used in low-level async code, and
* Preparing to support asynchronous (and synchronous) generators in the language.

## Motivation

This goal represents the next step on a multi-year program aiming to raise the experience of authoring "async Rust" to the same level of quality as "sync Rust". Async Rust is a crucial growth area, with 52% of the respondents in the [2023 Rust survey](https://blog.rust-lang.org/2024/02/19/2023-Rust-Annual-Survey-2023-results.html) indicating that they use Rust to build server-side or backend applications. 

### The status quo

Async Rust is the most common Rust application area according to our [2023 Rust survey](https://blog.rust-lang.org/2024/02/19/2023-Rust-Annual-Survey-2023-results.html). Rust is a great fit for networked systems, especially in the extremes:

* **Rust scales up**. Async Rust reduces cost for large dataplanes because a single server can serve high load without significantly increasing tail latency.
* **Rust scales down.** Async Rust can be run without requiring a garbage collector or [even an operating system][embassy], making it a great fit for embedded systems.
* **Rust is reliable.** Networked services run 24/7, so Rust's "if it compiles, it works" mantra means fewer unexpected failures and, in turn, fewer pages in the middle of the night.

Despite async Rust's popularity, using async I/O makes Rust significantly harder to use. As one Rust user memorably put it, "Async Rust is Rust on hard mode." Several years back the async working group collected a number of ["status quo" stories](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo.html) as part of authoring an async vision doc. These stories reveal a number of characteristic challenges:

* Common language features do not support async, meaning that [users cannot write Rust code in the way they are accustomed to](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_plays_with_async.html?highlight=closure#the-story):
  * [x] ~~[traits](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/alan_needs_async_in_traits.html)~~ (they [do now][afitblog], though gaps remain)
  * [x] ~~closures~~ ([stabilized](https://github.com/rust-lang/rust/pull/132706))
  * [ ] [drop](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/alan_finds_database_drops_hard.html)
  In many cases there are workarounds or crates that can close the gap, but users have to learn about and find those crates.
* Common async idioms have "sharp edges" that lead to unexpected failures, forcing users to manage [cancellation safety](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_gets_burned_by_select.html), subtle [deadlocks](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/aws_engineer/solving_a_deadlock.html) and other failure modes for [buffered streams](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_battles_buffered_streams.html). See also tmandry's blog post on [Making async Rust reliable](https://tmandry.gitlab.io/blog/posts/making-async-reliable/)).
* Using async today requires users to select a runtime which provides many of the core primitives. Selecting a runtime as a user [can be stressful](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_makes_their_first_steps_into_async.html#the-wrong-time-for-big-decisions), as the [decision once made is hard to reverse](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_wishes_for_easy_runtime_switch.html). Moreover, in an attempt to avoid "picking favorites", the project has not endorsed a particular runtime, making it [harder to write new user documentation](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/niklaus_wants_to_share_knowledge.html). Libraries meanwhile [cannot easily be made interoperable across runtimes](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_writes_a_runtime_agnostic_lib.html) and so are often written against the API of a particular runtime; even when libraries can be retargeted, it is difficult to do things like run their test suites to test compatibility. [Mixing and matching libraries can cause surprising failures.](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/alan_started_trusting_the_rust_compiler_but_then_async.html)

[afitblog]: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html
[embassy]: https://github.com/embassy-rs/embassy
[tokio]: https://tokio.rs/

### The next 6 months

#### Tell a complete story for async fn in traits

* Unblock AFIT in public traits by stabilizing RTN and implementable trait aliases (unblock tower 1.0)
* Ship 1.0 of the dynosaur crate, enabling dynamic dispatch with AFIT
* Stretch goal: Implement experimental support for async fn in `dyn Trait` in nightly

#### Improve ergonomics around `Pin`

* Ratify and implement an RFC for auto-reborrowing of pinned references
* Stretch goal: Discuss and implement a design for safe pin projection

#### Work toward asynchronous generators

* Have design meetings and ratify an RFC for a syntax for authoring synchronous iterators (e.g., synchronous generators)
* Stretch goal: Ratify an RFC for unsafe binders

In H2 we hope to tackle the following:

* RTN in type position
* Ratified RFC for syntactic support for asynchronous iteration

### The "shiny future" we are working towards

**Writing async code in Rust should feel just as expressive, reliable, and productive as writing sync code in Rust.** Our eventual goal is to provide Rust users building on async with

* the same core language capabilities as sync Rust (async traits with dyn dispatch, async closures, async drop, etc);
* reliable and standardized abstractions for async control flow (streams of data, error recovery, concurrent execution), free of accidental complexity;
* an easy "getting started" experience that builds on a rich ecosystem;
* good performance by default, peak performance with tuning;
* the ability to easily adopt custom runtimes when needed for particular environments, language interop, or specific business needs.

## Design axioms

* **Uphold sync Rust's bar for reliability.** Sync Rust famously delivers on the general feeling of "if it compiles, it works" -- async Rust should do the same.
* **Lay the foundations for a thriving ecosystem.** The role of the Rust org is to develop the rudiments that support an interoperable and thriving async crates.io ecosystem.
* **When in doubt, zero-cost is our compass.** Many of Rust's biggest users are choosing it because they know it can deliver the same performance (or better) than C. If we adopt abstractions that add overhead, we are compromising that core strength. As we build out our designs, we ensure that they don't introduce an "abstraction tax" for using them.
* **From embedded to GUI to the cloud.** Async Rust covers a wide variety of use cases and we aim to make designs that can span those differing constraints with ease.
* **Consistent, incremental progress.** People are building async Rust systems *today* -- we need to ship incremental improvements while also steering towards the overall outcome we want.

## Ownership and team asks

This section defines the specific work items that are planned and who is expected to do them. It should also include what will be needed from Rust teams. The overall owner of the effort is @tmandry. We have identified owners for subitems below; these may change over time.

### Overall program management

| Task                 | Owner(s) or team(s) | Notes |
|----------------------|---------------------|-------|
| AFIT story blog post | @tmandry            |       |

### Return type notation

| Task                        | Owner(s) or team(s)                                | Notes         |
|-----------------------------|----------------------------------------------------|---------------|
| Initial implementation      | @compiler-errors                                   | ![Complete][] |
| Author RFC                  | @nikomatsakis                                      | ![Complete][] |
| RFC decision                | ![Team][] [lang]                                   | ![Complete][] |
| Finished implementation     | @compiler-errors                                   | ![Complete][] |
| Standard reviews            | ![Team][] [types], [compiler]                      |               |
| Author stabilization report | @compiler-errors                                   |               |
| Author Reference PR         | TBD (@compiler-errors, @tmandry, or @nikomatsakis) |               |
| Review/revise Reference PR  | ![Team][] [lang-docs]                              | @nikomatsakis |
| Stabilization decision      | ![Team][] [lang], [types]                          |               |

### Unsafe binders

| Task                   | Owner(s) or team(s)       | Notes                   |
|------------------------|---------------------------|-------------------------|
| Initial implementation | @compiler-errors          | Stretch goal            |
| Author RFC             | @nikomatsakis             | Stretch goal            |
| RFC decision           | ![Team][] [lang], [types] | Stretch goal            |

### Implementable trait aliases

| Task               | Owner(s) or team(s)           | Notes    |
|--------------------|-------------------------------|----------|
| Author RFC         | @tmandry                      |          |
| Implementation     | @compiler-errors              |          |
| Standard reviews   | ![Team][] [types], [compiler] |          |
| RFC decision       | ![Team][] [lang], [types]     |          |

### `async fn` in `dyn Trait`

| Task                 | Owner(s) or team(s) | Notes         |
|----------------------|---------------------|---------------|
| Lang-team experiment | @nikomatsakis       | (Approved)    |
| Implementation       | @compiler-errors    | Stretch goal  |

### Pin ergonomics

| Task                 | Owner(s) or team(s) | Notes         |
|----------------------|---------------------|---------------|
| Implementation       | @eholk              |               |
| Author RFC           | @eholk              |               |
| Lang-team experiment | ![Team][] [lang]    | ![Complete][] |

### Trait for generators (sync)

| Task               | Owner(s) or team(s)          | Notes               |
|--------------------|------------------------------|---------------------|
| Implementation     | @eholk                       |                     |
| Author RFC         |                              |                     |
| RFC decision       | ![Team][] [libs], [lang]     |                     |
| Design meeting     | ![Team][] [lang]             | 2 meetings expected |

### Dynosaur 1.0

| Task             | Owner(s) or team(s) | Notes |
|------------------|---------------------|-------|
| Implementation   | @spastorino         |       |
| Standard reviews | @tmandry            |       |

### Definitions

Definitions for terms used above:

* *Discussion and moral support* is the lowest level offering, basically committing the team to nothing but good vibes and general support for this endeavor.
* *Author RFC* and *Implementation* means actually writing the code, document, whatever.
* *Design meeting* means holding a synchronous meeting to review a proposal and provide feedback (no decision expected).
* *RFC decisions* means reviewing an RFC and deciding whether to accept.
* *Org decisions* means reaching a decision on an organizational or policy matter.
* *Secondary review* of an RFC means that the team is "tangentially" involved in the RFC and should be expected to briefly review.
* *Stabilizations* means reviewing a stabilization and report and deciding whether to stabilize.
* *Standard reviews* refers to reviews for PRs against the repository; these PRs are not expected to be unduly large or complicated.
* *Prioritized nominations* refers to prioritized lang-team response to nominated issues, with the expectation that there will be *some* response from the next weekly triage meeting.
* *Dedicated review* means identifying an individual (or group of individuals) who will review the changes, as they're expected to require significant context.
* Other kinds of decisions:
    * [Lang team experiments](https://lang-team.rust-lang.org/how_to/experiment.html) are used to add nightly features that do not yet have an RFC. They are limited to trusted contributors and are used to resolve design details such that an RFC can be written.
    * Compiler [Major Change Proposal (MCP)](https://forge.rust-lang.org/compiler/mcp.html) is used to propose a 'larger than average' change and get feedback from the compiler team.
    * Library [API Change Proposal (ACP)](https://std-dev-guide.rust-lang.org/development/feature-lifecycle.html) describes a change to the standard library.

## Frequently asked questions

### Why work on synchronous generators if your goal is to support async?

There are three features we want that all interact quite heavily with each other:

* Sync generators
* Async generators
* Async iteration trait

Of the three, we think we are the closest to ratifying an RFC for synchronous generators. This should help clarify one of the major outstanding questions for the other two items; namely, the relation to pinning. With that out of the way, we should better be able to focus on the iteration trait and how well it works with async generators.

Focusing on pinning first also synergizes well with the efforts to improve the ergonomics of pinning.
