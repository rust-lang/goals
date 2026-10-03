# Prepare TAIT + RTN for stabilization

| Metadata         |                                                                                                    |
| :--              | :--                                                                                                |
| Contact | @traviscross                                                                                       |
| Funding contact | [Hexcat](https://hexcat.nl/) |
| Status           | Accepted                                                                                           |
| What and why     | Name opaque types and bound async return types so `async fn` in traits works with `Send` and `dyn` |
| Roadmap          | Just add async                                                                                     |
| Tracking issue   | [rust-lang/goals#646]                                                                 |
| Other tracking issues | [rust-lang/rust#63063], [rust-lang/rust#109417]                                               |
| Zulip channel    | [#wg-async][channel]                                                                               |
| [lang] champion  | @traviscross                                                                                       |
| [types] champion | @lcnr                                                                                              |


[channel]: https://rust-lang.zulipchat.com/#narrow/channel/187312-wg-async/

## Summary

Prepare TAIT (type alias impl trait) and return type notation (RTN) for stabilization together, giving Rust a coherent story for naming and bounding previously unnameable and unboundable types. TAIT lets users name opaque types like closures, async blocks, and complex iterators without boxing. RTN enables bounds like `T::method(..): Send`, solving the ["Send bound" problem][sb] and unblocking widespread use of async fn in traits. This goal also extends RTN to async closures via a new RFC. Full stabilization is blocked on the [next-gen trait solver](./next-solver.md) work and is intended to happen late this year.

**Needs contributor:** The majority of the impl work for TAIT and RTN has been done however the syntactic design for RTN and async closures is incomplete. We're seeking someone willing to help work on the RFC and explore the design space as well as to finalize impl details.

[sb]: https://smallcultfollowing.com/babysteps/blog/2023/02/01/async-trait-send-bounds-part-1-intro/

## Motivation

### The status quo

#### TAIT: naming the unnameable

Many Rust types cannot be written explicitly. Closures, async blocks, complex iterator chains, and nested combinators all produce anonymous types that have no surface syntax. Today, the only way to use these types in a type alias, struct field, or associated type is to box them (`Box<dyn Trait>`) or restructure code to avoid naming them entirely.

[Type alias impl trait][tait-rfc] (TAIT) solves this by letting you write `impl Trait` in type alias position:

```rust
// Name a complex iterator type without spelling it out
type OddIntegers = impl Iterator<Item = u32>;

fn odd_integers(start: u32, stop: u32) -> OddIntegers {
    (start..stop).filter(|i| i % 2 != 0)
}
```

The concrete "hidden type" is inferred by the compiler, so callers see only the trait bounds. This is valuable for hiding implementation details, simplifying complex type signatures, and, critically, implementing associated types in traits without boxing:

```rust
impl MyTrait for MyType {
    type Output = impl Iterator<Item = u32>;  // no Box<dyn> needed

    fn produce(&self) -> Self::Output {
        self.items.iter().filter(|x| x.is_valid()).copied()
    }
}
```

TAIT is available on nightly under the feature flag `type_alias_impl_trait`. See the [impl trait initiative][iti] for more details.

[tait-rfc]: https://rust-lang.github.io/rfcs/2515-type_alias_impl_trait.html
[iti]: https://rust-lang.github.io/impl-trait-initiative/

#### RTN: the Send bound problem

Async fn in traits (AFIT) has been stable since Rust 1.75, but when users attempt to use it in a public trait, they get a warning:

```rust
pub trait Foo {
    async fn bar();
}

// warning: use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
//  --> src/lib.rs:2:5
//   |
// 2 |     async fn bar();
//   |     ^^^^^
//   |
//   = note: you can suppress this lint if you plan to use the trait only in your own code, or do not care about auto traits like `Send` on the `Future`
//   = note: `#[warn(async_fn_in_trait)]` on by default
// help: you can alternatively desugar to a normal `fn` that returns `impl Future` and add any desired bounds such as `Send`, but these cannot be relaxed without a breaking API change
//   |
// 2 -     async fn bar();
// 2 +     fn bar() -> impl std::future::Future<Output = ()> + Send;
//   |
```

This warning is highlighting a problem known as the ["Send bound" problem][sb], which means that generic functions referencing a trait with async functions (or any `impl Trait`-returning function) cannot specify that this async function must return a `Send` future. This blocks ecosystem crates like [Tower](https://crates.io/crates/tower) from using AFIT in their public APIs.

Return type notation (RTN), proposed in [RFC #3654], solves this by letting you write:

```rust
fn spawn_service<S>(service: S)
where
    S: Service,
    S::call(..): Send,  // RTN: the future returned by `call` must be Send
{
    tokio::spawn(async move {
        service.call(request).await
    });
}
```

RTN has been fully implemented and is available on nightly under the feature flag `return_type_notation`.


#### RTN for async closures

RTN solves the Send bound problem for trait methods, but what about async closures? Consider this function:

```rust
async fn foo<F>(x: F)
where
    F: AsyncFn(&str) -> Option<()>,
{
    tokio::spawn(x("hello"));
}
```

This doesn't compile because `tokio::spawn` requires a `Send` future, but we have no way to express that `x()` returns one. The `F::method(..)` syntax doesn't work because there is no method name to reference.

The [async closure RFC][ext] mentioned a syntax like `F(..): Send`, but this wasn't widely discussed during that RFC process. We'd like to explore the syntactic design space, author an RFC, and implement this on nightly.

[ext]: https://rust-lang.github.io/rfcs/3668-async-closures.html#interaction-with-return-type-notation-naming-the-future-returned-by-calling

#### Why stabilize RTN and TAIT together?

TAIT and RTN are deeply connected: RTN can be used to model TAIT, and both deal with bounding and naming opaque types. A [stabilization PR][stab-pr] for RTN was opened but had to be closed because RTN's ability to name opaque types in arbitrary positions interacts with unresolved questions around TAIT and ATPIT (associated type position impl trait). Since RTN can model TAIT, stabilizing them separately risks locking in behaviors that would be awkward or wrong to change later. Rather than rushing RTN stabilization alone, this goal focuses on getting both ready so they can stabilize together, giving Rust a coherent story for bounding opaque types regardless of how they're expressed.

#### Dependency on the next-gen trait solver

Full stabilization is blocked on the [next-gen trait solver](./next-solver.md) work and is intended to happen late this year.

[stab-pr]: https://github.com/rust-lang/rust/pull/138424

### What we propose to do about it

We propose to prepare RTN for stabilization by:

- **Investigating what's needed from the lang side for TAIT**: RTN can be used to model TAIT, so it makes sense to stabilize them together. Both are very useful, and we need to understand what lang work TAIT requires.
- **Updating the stabilization report** to address the concerns raised in the closed stabilization PR, so that once TAIT is resolved, RTN stabilization can proceed without delay.
- **Extending RTN to async closures** via a new RFC and nightly implementation. This work is independent of the TAIT blocker and can proceed in parallel.

### Looking for contributors

We're looking to mentor one or more contributors on this goal. The work spans language design (RTN for async closures RFC), compiler implementation (nightly implementation of async closure RTN), and stabilization preparation (researching TAIT interactions, updating the stabilization report). This is a great opportunity to get involved in Rust's type system and async ecosystem. Reach out to TC if you're interested.

### Work items over the next year

| Task                                     | Owner(s)      | Notes                                      |
|------------------------------------------|---------------|--------------------------------------------|
| Research lang-side requirements for TAIT | @traviscross  | What's needed before RTN/TAIT can stabilize |
| Update stabilization report              |               | Address concerns from closed PR            |
| RFC for RTN on async closures            | @traviscross, @nikomatsakis | `F(..): Send` syntax, explore design space |
| Implement RTN for async closures         |               | After RFC acceptance                       |

## Team asks

| Team    | Support level | Notes                                         |
|---------|---------------|-----------------------------------------------|
| [lang]  | Medium        | RFC review, design discussions                 |
| [types] | Medium        | Stabilization report review, TAIT interactions |

## Help wanted

| Task | Experience level | Time investment |
|------|-----------------|-----------------|
| Explore designs for return type notation | TBD | TBD |
| Explore designs for async closures | TBD | TBD |
| Work on the RFC | TBD | TBD |
| Finalize the implementation | TBD | TBD |

## Frequently asked questions

### Why not stabilize RTN now?

The [stabilization PR][stab-pr] was closed because RTN allows naming opaque types in positions that weren't previously possible. As lcnr noted:

> RTN allows naming opaque types in arbitrary positions. This means we get nearly all implementation issues preventing TAIT, at least in theory.

The concern is that stabilizing RTN could lock in behaviors that would conflict with how TAIT/ATPIT need to work. Opaque types have a history of stabilizing features that "won't affect the future" and then being wrong. Rather than risk that, we want to get everything ready for stabilization and then stabilize once TAIT is resolved.

### How does this relate to the "Just add async" roadmap?

RTN is the key to making async fn in traits usable in practice. Without it, trait authors must choose between:
- Using `async fn` but preventing generic code from requiring `Send`
- Using explicit `-> impl Future<Output=T> + Send` but losing the ergonomics of `async fn`

RTN eliminates this tradeoff, letting patterns from sync Rust transfer naturally to async.
