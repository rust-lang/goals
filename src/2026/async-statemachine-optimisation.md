# Async statemachine optimisation

| Metadata            |                                       |
| :------------------ | :------------------------------------ |
| Contact             | @diondokter                           |
| Status              | Accepted                              |
| Roadmap             | Binary size reduction                 |
| Tracking issue      | [rust-lang/goals#623]                 |
| Zulip channel       | N/A                                   |
| [compiler] champion | @eholk                                |
| Funding contact     | [Tweede golf](https://tweedegolf.nl/) |

## Summary

Add optimisations to the coroutine MIR transform to elide or simplify the generated statemachines.

This will help both async futures and generators.

*Note: This has some overlap with [this other proposal](./async-future-memory-optimisation.md).
I'm not trying to usurp the issue. I was working on it incidentally at the same time and noticed very late that proposal already existed.*

## Motivation

### The status quo

Generated Rust futures (from async blocks and functions) are unoptimised. That is, they always naively generate a full statemachine even when it's not required.

Often on opt-level 3, the compiler is able to optimize out this statemachine. But when the tree of futures gets deep, this is no longer the case. The compiler struggles even more on opt-level s or z where even simple cases aren't optimized away.

Every statemachine also has a panicking branch. This litters the code with panics that makes the compiler's job a lot more difficult. Those panics will also pretty much never be run when a good functioning executor is used to poll the futures.

### What we propose to do about it

I want to come up with a list of optimisations we can do.
I've already thought of a couple.

A simple example:

```rust
// MIR for `foo::{closure#0}` 0 coroutine_resume
/* coroutine_layout = CoroutineLayout {
    field_tys: {},
    variant_fields: {
        Unresumed(0): [],
        Returned (1): [],
        Panicked (2): [],
    },
    storage_conflicts: BitMatrix(0x0) {},
} */

async fn foo() -> i32 {
    5
}
```

This generates a whole statemachine with 3 states. In the best case, LLVM can optimize this away.
But when this is deep in an async 'tree', the compiler can fail to optimize this away.
This leads to lost performance and binary size bloat. The latter of which is very noticeable on embedded.

Another example:

```rust
async fn bar() -> i32 {
    foo().await
}
```

Here `bar` gets its own future, just to poll the `foo` future.
Instead `bar` could 'become' the `foo` future which would save a statemachine.

I believe that if we keep stacking optimizations like these, we'll get some really nice results.

I've got 4 optimisations on my list so far. You can see them in the work items and in the draft blog post I'm writing (linked in FAQ).

### Work items over the next year

| Task                                                                                     | Owner(s)    | Notes                                   |
| ---------------------------------------------------------------------------------------- | ----------- | --------------------------------------- |
| Create (unstable?) compiler flag to replace future panics with returning `Poll::Pending` | @diondokter |                                         |
| Create an optimisation for futures with no awaits                                        | @diondokter |                                         |
| Create an optimisation for futures with one await (`bar` becomes `foo`)                  | @diondokter | Maybe generalisable to the 'last' await |
| Collapse states when multiple await points are the same                                  | @diondokter |                                         |

## Team asks

| Team       | Support level | Notes                                                                                                                                |
| ---------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| [compiler] | Medium        | Most will be review work, but pushing optimisations to the max will possibly touch on some controversial points that need discussion |

## Funding

| Purpose     | Cost    | Funded | Sponsor(s)          |
| ----------- | ------- | ------ | ------------------- |
| Contributor | $50,000 | Full   | OpenAI, AWS, +1 TBA |

## Target timeline

The duration of the project is 6 months. Starting from the agreed start date ("Month 1"), the timeline we're targeting is:

- Month 1-4: work on implementing the features
- Month 4-6: work is done, keep engaging the project to get it accepted and merged

The expected effort for the work is about 2.5 person-months, the long timeline is really due to not committing a full FTE and waiting on the project to deliberate and review the changes.

## Frequently asked questions

### Why this, why now?

I work as an embedded developer most of the time. We love to use async Rust, but as the firmware grows in size, the async bloat gets quite unwieldy.
For example, I developed a, async radio device driver. After I was done I wrote an example for it.

The example was about 35 KB in binary size. This seemed way too big, so I converted it to be mostly blocking code and the size dropped to about 17 KB.
This shows how much bloat async can add.

For a big customer we were running against the 900 KB limit of the hardware and had to do a lot of manual optimisations to the async code to make it fit.
I then looked into the MIR pass responsible and found it pretty much does no optimisations.

We're working on getting funding. In that process we've released two blog posts raising awareness:
- [debloat your async rust](https://tweedegolf.nl/en/blog/235/debloat-your-async-rust)
- [async rust never left the mvp state](https://tweedegolf.nl/en/blog/237/async-rust-never-left-the-mvp-state)

### What's your commitment?

Without funding other commercial work will sadly take priority. This means I can only commit up to half a day per week on most weeks.
There's interest from some companies to fund the work. When more becomes clear this section will be updated.

### How does this relate to the [other similar proposal](./async-future-memory-optimisation.md)?

We have similar goals, but a different philosofy.

I want to tackle the problem at the front:
- Stop generating statemachines that don't have to be there
- Make the compiler's job easier by removing panic paths and branches
- Make statemachines smaller

The other proposal tackles the problem at the back:
- Given there are statemachines, make them interact better with each other (real proper inlining)
- Research the options LLVM now provides

I think these can compliment each other. Optimisations done at the front will trickle down to the optimisations at the back.
Inlining a small statemachine should prove easier/more efficient than inlining a big statemachine.

Also, for code size you may not want to inline everything, just as with normal blocking code.
