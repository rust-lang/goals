# Evolving trait hierarchies

| Metadata         |                                    |
| :--              | :--                                |
| Point of contact | @cramertj                          |
| Status           | Proposed                           |
| Flagship         | Unblocking dormant traits          |
| Tracking issue   | [rust-lang/goals#393] |
| Zulip channel    |                                    |
| [lang] champion  | @cramertj                          |
| [types] champion | @oli-obk                           |


## Summary

Unblock the evolution of key trait hierarchies:

* Adding [`Receiver`](https://doc.rust-lang.org/std/ops/trait.Receiver.html)
  as a supertrait of [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html).
* Allow the `tower::Service` trait to be split into a non-`Sync` supertrait and a
  `Sync` (thread-safe) subtrait.

The design should incorporate the feedback from the
[Evolving trait hierarchies](https://hackmd.io/6JId0y8LTyCzVMfZFimPqg)
language design meeting. The design should also set the stage for future changes to allow for
the more general case of splitting items out into supertraits.

## Motivation

Two significant motivating cases are discussed in [RFC 3437 "Implementable trait alias"](https://github.com/Jules-Bertholet/rfcs/blob/implementable-trait-alias/text/3437-implementable-trait-alias.md#deref--receiver--deref) under the heading "splitting a trait".

### Conceptual supertrait: `Deref: Receiver`

The `Deref` trait currently looks like this:

```rust
pub trait Deref {
    type Target: ?Sized;

    fn deref(&self) -> &Self::Target;
}
```

More recently, the `arbitrary_self_types` feature has motivated a more general `Receiver` trait:

```rust
pub trait Receiver {
    type Target: ?Sized;
}
```

Ideally, `Receiver` would be a supertrait of `Deref`:

```rust
pub trait Receiver {
    type Target: ?Sized;
}

pub trait Deref: Receiver {
    fn deref(&self) -> &Self::Target;
}
```

but this cannot be done today without a breaking change.

More details are in this Pre-RFC: [Supertrait associated items in subtrait impl](https://hackmd.io/@rust-for-linux-/SkucBLsWxl)

### Conceptual supertrait: `Iterator: LendingIterator`

Similarly, every type that implements today's `Iterator` trait could also (conceptually) be a `LendingIterator`:

```rust
pub trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

#### Missing or misnamed parent trait items

Note that, unlike `Deref` and `Receiver`, the names and signatures of the associated items in `LendingIterator` do not match those in `Iterator`. For existing `Iterator` types to implement `LendingIterator`, some bridge code between the two implementations must exist.
### Conceptual supertrait: relaxed bounds

A common practice in the `async` world is to use [the `trait_variant` crate](https://docs.rs/trait-variant/latest/trait_variant/) to make two versions of a trait, one with `Send` bounds on the futures, and one without:

```rust
#[trait_variant::make(IntFactory: Send)]
trait LocalIntFactory {
    async fn make(&self) -> i32;
    fn stream(&self) -> impl Iterator<Item = i32>;
    fn call(&self) -> u32;
}

// `trait_variant` will generate a conceptual subtrait:

trait IntFactory: Send {
    fn make(&self) -> impl Future<Output = i32> + Send;
    fn stream(&self) -> impl Iterator<Item = i32> + Send;
    fn call(&self) -> u32;
}
```

In this example, a type that implements `IntFactory` also satisfies the requirements for `LocalIntFactory`, but additionally guarantees that the returned types are `Send`.

### The status quo

Today's solutions include:

### Add a supertrait

#### Pros
* Matches the conceptual pattern between the traits: one clearly implies the other.
* The "standard" Rust way of doing things.

#### Cons
* Breaking change.
* Requires implementors to split out the implementation into separate `impl` blocks for each trait.

### Add a blanket impl

Rather than declaring `trait A: B`, one can create sibling traits with a blanket impl:

```rust
trait Supertrait {
  // supertrait items
}

trait Subtrait {
  // supertrait items + subtrait items
}

impl<T: Subtrait> Supertrait for T {
  // impl supertrait items using subtrait items
}
```

#### Pros
* Backwards compatible to introduce `Supertrait`

#### Cons

* Middleware impls are impossible! We'd like to write:

```rust
struct Middeware<T>(T);

impl<T: Supertrait> Supertrait for Middleware<T> { ... }

impl<T: Subtrait> Subtrait for Middleware<T> { ... }
```

but this overlaps with the blanket impl, and is rejected by the Rust compiler! This is a critical issue for `async` bridge APIs such as tower's `Service` trait, which wants to provide wrappers which implement the `trait_variant`-style `Send`-able when the underlying type implements the `Send` version (see [these notes from a previous design meeting](https://hackmd.io/rmN25qziSHKT4kv-ZC8QPw)).

Other nits:
* The directionality of the impl is less clear.
* Every shared item has two names: `<T as Supertrait>::Item` and `<T as Subtrait>::Item`. Relatedly, the bridge impl must exist even if items are identical.

### Provide no bridging

Another alternative is to provide two totally separate traits with no bridging, requiring users to manually implement both versions of the trait:

```rust
trait Supertrait { ... }
trait Subtrait { ... }

struct Impl { ... }

impl Supertrait for T { ... }
impl Subtrait for T { ... }
```

#### Pros

* Backwards compatible
* Middleware can be written to bridge either impl

#### Cons

* Requires duplication
* Requires users to restate bounds
* Existing code which implements `Subtrait` cannot be used as `Supertrait`, so APIs which require `Subtrait` cannot be relaxed to `Supertrait`

### The next 6 months

In the next six months, we aim to ship a solution which addresses the `Service` and `Receiver` use-cases
by allowing trait impls to implement supertraits *if* the impl itself contains a definition of an item
from the supertrait.

Traits will have to opt their impls into this behavior, possibly through the use of a keyword.
`auto` is used below as an example keyword:

```rust
// Library code:
trait Subtrait {
    fn supertrait_item();
    fn subtrait_item();
}
// User code:
impl Subtrait for MyType {
    fn supertrait_item() { ... }
    fn subtrait_item() { ... }
}

// -- can become --

// Library code:
trait Supertrait {
    fn supertrait_item();
}
trait Subtrait: auto Supertrait {
    fn subtrait_item();
}
// User code is unchanged from above, no separate `Supertrait`
// impl required
impl Subtrait for MyType {
    fn supertrait_item() { ... }
    fn subtrait_item() { ... }
}
```

### The "shiny future" we are working towards

In the future, we'd like it to be backwards-compatible for traits to split out arbitrary, possibly
defaulted items into supertraits.

This will be challenging, as it won't be obvious syntactically whether an impl intends to provide a
supertrait impl-- some degree of coherence / overlap resolution will be required. However, this feature
will provide library authors a great deal of flexibility while allowing for more ergonomic end-user
implementations.

## Ownership and team asks

| Task                         | Owner(s) or team(s)  | Notes                                                                |
| ---------------------------- | -------------------- | -------------------------------------------------------------------- |
| Discussion and moral support | ![Team][] [lang]     |                                                                      |
| Discussion and moral support | ![Team][] [compiler] |                                                                      |
| Discussion and moral support | ![Team][] [types]    |                                                                      |
| Author RFC                   | @cramertj            |                                                                      |
| Implementation               | @cramertj & others   |                                                                      |
| Stabilization decision       | ![Team][] [libs]     | Stabilizing `Receiver`. Unblocked by implementation.                 |
| Stabilization decision       | ![Team][] [lang]     | Stabilizing `arbitrary_self_types`. Unblocked by new `Receiver` API. |

### Definitions

For definitions for terms used above, see the [About > Team Asks](https://rust-lang.github.io/goals/about/team_asks.html) page.

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
