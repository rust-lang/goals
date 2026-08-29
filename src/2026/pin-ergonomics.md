# Continue Experimentation with Pin Ergonomics

| Metadata            |                                    |
|:--------------------|:-----------------------------------|
| Contact    | @frank-king                        |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#389] |
| Zulip channel       | N/A                                |
| [compiler] champion | @oli-obk                           |
| [lang] champion     | @traviscross                       |
| [types] champion    | @oli-obk                           |


## Summary

Continue experimenting with and fleshing out the design and semantics for the pin ergonomics experiment.

## Motivation

Several major Rust features, such as `async` and generators, depend on the ability to ensure data will not move in memory.
The way Rust achieves this is through the `Pin` wrapper, but pinning has notoriously poor ergonomics.
Exploring how the ergonomics can be improved by integrating pinning into the language better could help unblock advancements in other Rust features.

### The status quo

Pinning exists but few people like it.
We have an experiment for improving the ergonomics around pinning and some initial PRs have landed, but we would like to build more sustained momentum on it.

### Work items over the next year

We need to finish the borrow checker support, coersion and auto borrowing. It seems the [Reborrow traits goal](https://goals.rust-lang.org/2025h2/autoreborrow-traits.html) might help/solve auto borrowing?

But there's also broader work around describing the new borrowchk behavior and gathering feedback on it. In particular, the `&pin mut` borrow works differently from a normal `&mut` borrow.

A regular `&mut` takes a temporary exclusive access to the original place and releases it once the borrow's lifetime expired.

But a `&pin mut` keeps the pinned effect *even after the borrow's lifetime expired*. Once you pin a place, it stays pinned for the duration of the program.

This is inconsistent and user feedback will be really important here -- both in terms of exploring the impact of this fully and how teachable this is.

| Task                                     | Owner(s)    | Notes               |
|------------------------------------------|-------------|---------------------|
| borrowchk for `&pin .. place`            | @frank-king | Is this still TODO? |
| `Drop::pin_drop`                         | @frank-king |                     |
| `&pin .. T <-> &[mut] T` coercion        | @frank-king |                     |
| Auto borrowing                           | @frank-king |                     |
| Collect borrowchk's design user feedback | @frank-king |                     |
| update the RFC                           | @frank-king |                     |
| Call for testing blog post               | @frank-king |                     |
| Stabilization?                           | @frank-king |                     |

## Team asks

| Team       | Support level | Notes           |
|------------|---------------|-----------------|
| [compiler] | Medium        | Reviews         |
| [lang]     | Medium        | Design meeting? |
| [types]    | Medium        | Reviews         |
