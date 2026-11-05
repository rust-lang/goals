# Iterator exploration

| Metadata            |                                                                       |
| ------------------- | --------------------------------------------------------------------- |
| Contact             | TBD                                                                   |
| Status              | Proposed                                                              |
| What and why        | Investigate long-term changes to the standard library `Iterator` APIs |
| [compiler] champion | TBD                                                                   |
| [lang] champion     | TBD                                                                   |
| [libs] champion     | TBD                                                                   |
| [opsem] champion    | TBD                                                                   |
| [types] champion    | TBD                                                                   |

## Summary

Research what a new version of the `Iterator` APIs would look like, reporting back what the bounds of such a change would look like and which language features would be most appropriate.

## Motivation

### The status quo

The [`Iterator`] trait and its extended family is one of the most ubiquitous and important features of the Rust standard library, and its design also leaves *a lot* to be desired, with lessons learned after over a decade of Rust 1.0 and new features.

[`Iterator`]: https://doc.rust-lang.org/nightly/std/iter/trait.Iterator.html

The number of weird design quirks are impossible to fully enumerate, but, here are a couple:

* Iterator items are *not* lifetime-generic, meaning that they cannot borrow values from within the iterator itself. This leads to issues like [`std::env::args`] having to allocate new strings for every single argument.
* Size hints for iterators have dubious usefulness, and they don't offer a way to represent infinite iterators. It's unclear how users should respond to size hints and what purpose they serve.
* A number of iterator properties have to have additional "unsafe" traits that allow people to rely on their implementation being correct for soundness.
* Iterator adapters often require storing fields for all possible iterator properties instead of only adapting to have the ones they need.
* Async iterators are not built-in by default, and for all intents and purposes, we need to duplicate everything between async and sync iterators.
* Several traits exist which are "iterator-like," such as the various [formatting traits](https://doc.rust-lang.org/nightly/std/fmt/index.html) and [I/O traits](https://doc.rust-lang.org/nightly/std/io/trait.Read.html), which currently require having their own collection of "pseudo-iterator adapters" but cannot reuse the existing adapters.
* And more…

[`std::env::args`]: https://doc.rust-lang.org/nightly/std/env/fn.args.html

Additionally, [entire language features](https://github.com/rust-lang/rfcs/pull/3530) have been proposed just to help wrangle the [`Iterator`] trait better, due to the way that default implementations and specialization work in Rust. Clearly, something needs to be done.

### What we propose to do about it

With [edition redirects](https://github.com/rust-lang/rust/pull/160227) on the horizon, it's time to start seriously considering what a proper change to [`Iterator`] would look like, and what language features would strictly block these changes, and which ones shouldn't.

To be clear, this goal *is not* to actually redesign [`Iterator`], but simply to apply further bounds on what such a redesign would look like, what the shortest path to a useful redesign would look like, and what the blockers for such an implementation would look like. There are several different ways to tackle all the problems with [`Iterator`], but we don't know what would be most useful for the ecosystem and what the tradeoffs are between making redesigns sooner and having to maintain more than just one redesign of these traits due to edition guarantees. (With editions, we need to keep a copy of every version for every edition forever, which is a notable maintenance burden.)

Just looking at the simple example of `size_hint`, there are already a lot of different things that could be noted:

* Some parts of the size hint can be inferred statically, like the fact that infinite iterators would always return `Some` instead of `None`. Perhaps this might be a valid reason for these iterators to return `Result<Item, !>`, so that we can guarantee that the `None` state (in this case, `Err`) is unreachable. This would allow completely removing branches that check whether the iterator has ended at compile time.
* Even if the length of an iterator is nontrivial to compute, if it can be computed at compile time, it might be worth it for code to be able to use the exact computed amount anyway.
* Computed estimates for size depend on use case: for example, on a memory-constrained system, assuming that a UTF-16 string takes up twice the length of its UTF-8 counterpart may be unacceptable, and such a hint would be less useful.
* Being able to rely on the size for correctness, like being able to directly index into a slice and eliminate bounds checks, would require explicitly marking the size hint as such. This might be better suited to a general "overconstraining" `unsafe impl` feature, rather than choosing a solution specific to `Iterator`. (For example, relying on the correctness of `Ord` for safety could be useful.)

It's very clear that such an redesign could be subject to endless scope creep, and rather than planning to explicitly solve all of these problems now, it would be good to at least perform an extensive investigation of ecosystem usage, performance and implementation bottlenecks, optimization issues, language design, and so on to determine what the path forward would be.

Note that the path forward *can* include both blocking and non-blocking language features. For example, something like effect generics (being able to generalize over async-ness) could be useful for async iterators, but might not block an MVP iterator design, allowing future redesigns.

### Work items over the next year

| Task                    | Owner(s) | Notes                                                                                                  |
| ----------------------- | -------- | ------------------------------------------------------------------------------------------------------ |
| Ecosystem investigation | None yet | What features do ecosystem crates want, what do they currently use?                                    |
| Compiler investigation  | None yet | What are the current performance/optimization bottlenecks that are relevant? How easy are they to fix? |
| Language investigation  | None yet | What language features would provide the best path to success here?                                    |
| Safety investigation    | None yet | Are the current designs people are using, or proposing, actually sound? What issues show up today?     |

## Team asks

The team asks here are weird, since nothing actually needs to be *implemented* per se, but the investigation should involve close contact with teams and their roadmaps. Effectively, the point of the investigation is to figure out where this change would fit into existing plans, and to plan around it for a more concrete proposal.

| Team        | Support level | Notes                                                               |
| ----------- | ------------- | ------------------------------------------------------------------- |
| [compiler]  | Large         | Codegen investigation                                               |
| [lang]      | Large         | Type system feature investigation                                   |
| [libs]      | Large         | API/ecosystem investigation                                         |
| [opsem]     | Large         | Safety audits (is what people are doing actually sound? can it be?) |
| [types]     | Large         | Language feature investigation                                      |

## Funding

TBD. Scope should be defined better within the proposal.

| Purpose | Cost    | Funded | Sponsor(s) |
| ------- | ------- | ------ | ---------- |
| ????    | Unknown | No     |            |
