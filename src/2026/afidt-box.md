# Native async fn dynamic dispatch in traits

| Metadata            |                                        |
| :--                 | :--                                    |
| Contact             | @spastorino                            |
| Status              | Accepted                               |
| What and why        | Enable dyn dispatch for async traits   |
| Timespan            | 2026-2027                              |
| Roadmap             | Just add async                         |
| Tracking issue      | [rust-lang/goals#625]                  |
| Highlight           | Async and ergonomic RC                 |
| Zulip channel       | [#wg-async][channel]                   |
| Funding contact     | @spastorino                            |
| [lang] champion     | @jackh726                              |
| [types] champion    | @jackh726                              |
| [compiler] champion | @spastorino                            |


[channel]: https://rust-lang.zulipchat.com/#narrow/channel/187312-wg-async/

## Summary

Design and implement native support for `async fn` methods invoked through `dyn` trait objects.

## Motivation

### The status quo

Async fn in traits (AFIT) has been stable since Rust 1.75, but traits with async methods are not dyn-compatible. If you write:

```rust
trait HttpClient {
    async fn fetch(&self, url: &str) -> Response;
}
```

You cannot use `&dyn HttpClient`. The compiler rejects it because async methods return opaque future types whose size isn't known at compile time, and dyn dispatch requires the caller to allocate space for the return value without knowing the concrete type.

Today, developers work around this by:
- **Using static dispatch only**: Generics everywhere, increasing compile times and binary size
- **Manual desugaring**: Write `-> Pin<Box<dyn Future<Output = Response> + Send + '_>>` and lose the ergonomics of `async fn`
- **Proc macros**: Use crates like [async-trait] or [dynosaur] that transform your code

[async-trait]: https://crates.io/crates/async-trait
[dynosaur]: https://crates.io/crates/dynosaur

#### The broader problem is complex

Making `dyn Trait` work with unsized return types is a deep problem that has been [explored extensively][dyn-async-series].
The caller must provide storage for the returned value, but doesn't know its size.
There are many options one might wish to use, with stack allocation and boxing being the most obvious.
The [in-place initialization goal](./in-place-init.md) is exploring the design space here for a fully general solution that allows the caller to have total control.
But that design work requires time, and the goal for 2026 is only to settle on a *specific design*, not necessarily to implement it or even stabilize it! In the meantime, Rust's support for async-fn-in-trait is only usable in narrow circumstances.

[dyn-async-series]: https://smallcultfollowing.com/babysteps/series/dyn-async-traits/

#### Boxing is the simplest approach and it's what many users will want

The widespread use of the [async-trait] crate demonstrates that boxing is perfectly acceptable for many applications.
`async-trait` transforms `async fn` into a fn that returns `Pin<Box<dyn Future + Send>>`, allocating on every call.
Despite this cost, the crate has been downloaded millions of times because for most server and application code, the allocation overhead is negligible compared to the I/O being performed.

The problem with `async-trait` isn't that it boxes.
It's that it modifies the *trait definition*, forcing all implementors to box on every call.
This means library authors can't use it for public traits where some users need static dispatch (no allocation) while others want dyn dispatch.

What we want is for the *call site* to decide whether to box.
A library like Tower could define its `Service` trait using native `async fn`, implementors would write normal async code without any boxing, and generic code using `T: Service` would have zero allocation overhead.
But users who need `dyn Service` could opt into boxing at the call site.
Once the in-place initialization work proceeds, that same trait would support other allocation strategies too.
But boxing unblocks the ecosystem now.

### What we propose to do about it

The work proceeds in two phases: first reforming dyn trait compatibility and then building the compiler support for async dispatch through trait objects and putting the caller in control of how (and whether) allocation happens.

#### Phase 1: Reforming dyn traits (4 months)

Today Rust is overly conservative about which traits can be used as dyn objects: if a trait has even one method that doesn't fit the current dynamic dispatch model, the entire trait is locked out of dynamic use.
Phase 1 reworks this to be precise rather than all or nothing.
A trait becomes usable dynamically while only the genuinely incompatible methods are left out of the runtime dispatch mechanism, instead of disqualifying the whole trait.

This method-scope dyn compatibility is foundational and not specific to async.
It benefits a broad range of traits, and it is the groundwork everything else depends on.
Concretely, `async fn` (and `-> impl Trait` methods in general) are not dyn compatible in the classic sense.
Phase 1 makes the trait dyn-compatible anyway, exposing the non-dispatchable methods only through the new mechanism.
The phase includes authoring the RFC and shepherding it through Rust's design teams.

#### Phase 2: Async through dynamic dispatch (8 months)

This is the headline capability: making `async fn` methods work through trait objects.
The core difficulty is that an async method returns a future whose size is not known at the point where dynamic dispatch happens, which is precisely why today's workaround heap-allocates on every call.
Phase 2 builds the compiler support to handle this natively and to put the *caller* in control of how, and whether, that allocation occurs.

There are two approaches to the underlying mechanism, one based on adjusting the low-level calling convention, the other on teaching the runtime dispatch table how large the future is so that a single, explicit allocation can be made.
Prototyping and choosing between them is part of the work, and part of why this is the longest phase.

While the feature matures, it is exposed through a deliberately minimal, nightly-only helper:

```rust
async fn fetch_data(client: &dyn HttpClient, url: &str) -> Response {
    std::preview::dyn_box!(client.fetch(url)).await
    //            --------
    //
    // Boxes the returned future so it can be awaited through `dyn`.
}
```

This lets real users exercise the capability without the project prematurely committing to final, polished syntax.
That polish can come later, on its own track, and the details of a more general form will depend on the outcome from the [in-place initialization](./in-place-init.md) exploration currently taking place.
This phase also carries the formal proposal and the consensus building with Rust's [lang] team needed to get it accepted.

For more details and a broader look, see the [box, box, box][box-post] blog post.

[box-post]: https://smallcultfollowing.com/babysteps/blog/2025/03/24/box-box-box/

### Work items over the next year

| Task                                          | Owner(s)     | Notes |
| --------------------------------------------- | ------------ | ----- |
| **Phase 1:** RFC for method-scope dyn compat  | @jackh726    |       |
| **Phase 1:** Implement method-scope dyn compat| @spastorino  |       |
| **Phase 2:** Prototype dispatch mechanism     | @spastorino  |       |
| **Phase 2:** Implement `dyn_box!` helper      | @spastorino  |       |
| **Phase 2:** RFC and [lang] consensus         | @jackh726    |       |
| Reviews                                       | @jackh726    |       |
| Documentation                                 | @spastorino  |       |

## Team asks

| Team       | Support level | Notes                                       |
| ---------- | ------------- | ------------------------------------------- |
| [lang]     | Medium        | RFC decision                                |
| [compiler] | Medium        | Implementation review                       |
| [types]    | Small         | May have changes to dyn-compatibility rules |

## Funding

The duration of the project is 12 months of full-time work (~1,920 hours) split between 2 contributors.

- Month 1-4 (Phase 1 — reforming dyn traits):
  - Author the RFC for method-scope dyn compatibility and shepherd it through the [lang], [types], and [compiler] teams.
  - Implement method-scope dyn compatibility on nightly so that incompatible methods no longer disqualify the whole trait.
- Month 5-12 (Phase 2 — async through dynamic dispatch):
  - Prototype both dispatch mechanisms (adjusting the calling convention vs. teaching the vtable the future's size) and choose between them.
  - Implement the nightly-only `std::preview::dyn_box!` helper so users can invoke `async fn` through `dyn` and explicitly box the returned future.
  - Author the RFC and build [lang] team consensus for async dispatch through trait objects.
  - Documentation.

| Purpose      | Cost | Funded | Sponsor(s)                     |
|--------------|------|--------|--------------------------------|
| Contributors | Ask  | Partial| [AWS](https://aws.amazon.com/) |

## Frequently asked questions

### Why explicit `dyn_box!` instead of implicit boxing?

Explicit is better here. Boxing has runtime cost (allocation), and Rust's philosophy is to make costs visible.
Writing `dyn_box!(...)` at the call site lets you see exactly where allocations happen, which matters for performance-sensitive async code.

### Why a `std::preview::dyn_box!` macro rather than final syntax?

The macro is deliberately minimal and nightly only.
It lets real users exercise it while the feature matures, without the project prematurely committing to final, polished syntax.
That polish, for example a more general operator that could eventually replace `Box::new`, can come later on its own track, once the [in-place initialization](./in-place-init.md) exploration settles the broader design.

### What about the other allocation strategies?

The status quo section describes several options: boxing, in-place initialization, inline storage, and custom allocators.
This goal focuses on boxing because it's the option we're most confident about.
It's proven by `async-trait`, works everywhere with an allocator, and is simplest to implement.

Other strategies can be added later with different syntax.
For example, inline storage might use `.inline::<N>` or similar.
The key insight is that we don't need to solve everything at once.

### Could this generalize beyond async?

Yes. The mechanism could potentially work anywhere you would use `Box::new`, but we don't want to figure out the full desugaring yet.
The nightly helper is scoped to method call position (`dyn_box!(foo.bar())`) for RPITIT methods.
That includes `async fn`, explicit `-> impl Future`, but also `-> impl Iterator` and similar patterns.

### Why not just use the dynosaur crate?

The dynosaur crate is a good workaround, but native language support would:
- Avoid proc macro complexity and compile-time overhead
- Provide better error messages
- Enable optimizations the compiler can't do through macro-generated code
- Make the pattern discoverable and standard

### How does this relate to the "Just add async" roadmap?

Dynamic dispatch is a core Rust pattern.
In sync code, you can freely use `&dyn Trait` for most traits.
The "Just add async" theme is about making async code work like sync code.
With `.box`, you can use `&dyn Trait` with async methods.
You just need to explicitly box the future, which is a reasonable cost for the flexibility of dynamic dispatch.
