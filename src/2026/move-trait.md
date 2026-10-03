# Immobile types and guaranteed destructors

| Metadata              |                                                                                                      |
| :--                   | :--                                                                                                  |
| Contact      | @lcnr                                                                                            |
| Funding contact | [Hexcat](https://hexcat.nl/) |
| Status                | Accepted                                                                                             |
| What and why          | Let types opt out of being moved or forgotten, enabling scoped spawn, async drop, and pin-by-default |
| Timespan              | 2026-2027                                                                                            |
| Roadmap               | [Just add async](./roadmap-just-add-async.md)                                                        |
| Roadmap               | [Rust for Linux](./roadmap-rust-for-linux.md)                                                        |
| Tracking issue        | [rust-lang/goals#635]                                                                   |
| Other tracking issues | [rust-lang/rust#149607]                                      |
| Zulip channel         | [#t-lang/move-trait](https://rust-lang.zulipchat.com/#narrow/channel/549962-t-lang.2Fmove-trait)     |
| [types] champion      | @lcnr                                                                                                |
| [lang] champion       | @jackh726                                                                                            |


## Summary

We propose to introduce new traits that describe what operations are possible on a type. Today Rust assumes all types can be moved (relocated in memory) and forgotten (via `mem::forget`). We will introduce traits like `Move` and `Forget` that make these capabilities explicit, allowing types to opt out. This follows the precedent set by the [Sized hierarchy work](./scalable-vectors.md), which relaxes the assumption that all types have a compile-time-known size. We will implement MVPs in the compiler, write RFCs, and validate viability through real-world testing in the Linux Kernel.

## Motivation

### The status quo

Rust has historically assumed that all values can be moved (relocated in memory) and forgotten (via `mem::forget`, without running destructors). These assumptions are baked into the language: assignment moves values, and `mem::forget` is safe. But some types need to opt out of these capabilities:

**Immobile types:** A lot of async futures want to be self-referential, but self-referential types can't be safely moved. The current solution is `Pin`, which encodes immovability as a property of *places* rather than *types*. This leads to significant complexity. As [The Safe Pinned Initialization Problem](https://rust-for-linux.com/the-safe-pinned-initialization-problem) describes, `Pin` struggles to safely encode self-referential types in systems like the Linux kernel.

**Guaranteed destructors:** Some types need their destructors to run. A `Transaction` type might require `commit()` or `rollback()` before cleanup. A scoped task handle must join before the scope exits. But `mem::forget` is safe, so Rust can't guarantee destructors run. This blocks patterns like safe scoped spawn for async, where the spawned task borrows from the parent scope.

### What we propose to do about it

We propose to generalize Rust's type system with new auto-traits that describe what operations are possible on a type. The framing is positive: traits represent capabilities. At the base layer, types may have no special capabilities. We then layer on the things we need:

- **`Move`**: The type can be relocated in memory.
- **`Destruct`**: The type can be implicitly dropped (destructor runs when it goes out of scope).
- **`Forget`**: The type can be forgotten via `mem::forget` without running its destructor.

This follows the precedent set by the [Sized hierarchy](./scalable-vectors.md) work. Just as that work relaxes "all types have compile-time-known size" to support scalable vectors, this work relaxes "all types can be moved" and "all types can be forgotten."

**The `Move` trait** encodes movability as a property of types rather than places:

```rust
#[lang = "move"]
unsafe auto trait Move {}
```

Types implementing `!Move` cannot be moved and must keep a stable address for their entire existence. This is simpler than `Pin` because immovability is a type property, not a place property. Construction of `!Move` types will rely on work from [#t-lang/in-place-init](https://rust-lang.zulipchat.com/#narrow/channel/528918-t-lang.2Fin-place-init).

**The `Forget` trait** lets types opt out of being forgettable:

```rust
// Types implementing !Forget must have their destructors run
impl !Forget for ScopedTaskHandle {}
```

With `!Forget`, we could build safe scoped spawn: the handle's destructor joins the task, and because the handle can't be forgotten, the join is guaranteed. This unblocks patterns that are currently impossible in safe Rust.

### Work items over the next year

#### `Move` trait

Let types opt out of being relocated in memory, encoding immovability as a type property rather than a place property.

| Task                                             | Owner(s)         | Notes                                                                                                                              |
| ------------------------------------------------ | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Compiler implementation for `Move`               | @lcnr and @nia-e |                                                                                                                                    |
| Write the `Move` RFC                             | @yoshuawuyts     |                                                                                                                                    |
| Test in Linux kernel                             | @BennoLossin     | RfL is an important Rust user which uses a lot of self-referential data structures.                                                |
| Test interactions between `Iterator` and `!Move` | @yoshuawuyts     | It's important to prove that generator-based effects can be desugared to `impl Trait + !Move` so they can support self-references. |

#### Guaranteed destructors

Explore letting types opt out of `mem::forget`, enabling patterns like safe scoped spawn for async.

| Task                                             | Owner(s)         | Notes                                                                                                                              |
| ------------------------------------------------ | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Design exploration for guaranteed destructors    | @nikomatsakis    | Explore trait hierarchy options and interaction with existing features                                                             |

What is concretely out of scope for this year is anything related to changing or
updating the `Future` trait. This is the only stable trait in Rust which depends
on `Pin`, and would need a migration story to be able to use `Move`. However depending on `Pin` is not the only shortcoming `Future` has ([1] + [2] +  10 more issues), and so fixing the `Future` trait is best treated as a standalone project.

[1]: https://blog.yoshuawuyts.com/the-waker-allocation-problem/
[2]: https://blog.yoshuawuyts.com/gen-auto-trait-problem/

## Team asks

| Team    | Support level | Notes                                        |
| ------- | ------------- | -------------------------------------------- |
| [lang]  | Large         | Design session needed to work through design |
| [types] | Large         | Involved in implementation + review          |

## Frequently asked questions

### How does this relate to the Sized hierarchy work?

The [Sized hierarchy](./scalable-vectors.md) work establishes the pattern: Rust can relax assumptions that were previously universal by introducing trait hierarchies that let types opt out. That work relaxes "all types have compile-time-known size" to support scalable vectors and extern types. This goal applies the same pattern to "all types can be moved" and "all types can be forgotten."

### How does this relate to the "pin ergonomics" initiative?

This work is an alternative to [Project Goal 2025H2: Continue Experimentation with Pin Ergonomics](https://github.com/rust-lang/goals/blob/main/src/2025h2/pin-ergonomics.md), which includes the following extensions:

 - A new item family `pin` in lvalues, e.g. `&pin x`, `&pin mut x`, `&pin const x`.
 - A one-off overload of Rust's `Drop` trait, e.g. `fn drop(&pin mut self)`.
 - A new item kind `pin` in patterns, e.g. `&pin <pat>`.

Notably this work does not solve [pin's duplicate definition
problem](https://blog.yoshuawuyts.com/why-pin/), meaning that even with these extentions we still end up with `Trait` and `PinnedTrait` variants of existing
traits. The `Drop` trait being the exception to this, since the initiative is proposing to special-case it using a one-off overload.

Rather than trying to change the language to make `Pin` work, we believe the
problem is with `Pin` and we should improve the way immovable types are encoded
in Rust instead. With the eventual goal to deprecate `Pin` in Rust entirely.

Because Rust promises to stay backwards-compatible forever, making `pin` a
language-item on par with `&` and `mut` is something we'll forever need to keep
supporting. Given our eventual goal is to deprecate `Pin`, we do not believe
that we should make `pin` a part of the language. Which is why `Move` is not just a complimentary proposal, but intended as an alternative.

### What enables safe scoped spawn?

Safe scoped spawn requires guaranteed destructors. The pattern: spawn returns a handle whose destructor joins the task. If you could `mem::forget` the handle, the task could outlive the scope and access dangling references. With `!Forget`, the handle's destructor is guaranteed to run, making the pattern safe. This is one of the key motivations for the guaranteed destructors portion of this goal.

### Where can I read more about this design space?

Several blog posts explore this area:

- [Move, Destruct, Leak](https://smallcultfollowing.com/babysteps/blog/2025/10/21/move-destruct-leak/) explores the trait hierarchy for destructors and forgetting. Note that unlike this post, we don't believe `Move` should be a supertrait of `Destruct`. It's useful to have types that can be destructed/dropped but not moved (e.g., self-referential types that need cleanup).
- [Must move types](https://smallcultfollowing.com/babysteps/blog/2023/03/16/must-move-types/) introduces the concept of types that force callers to take specific actions.
- [Ergonomic Self-Referential Types for Rust](https://blog.yoshuawuyts.com/self-referential-types/) and [its follow-up](https://blog.yoshuawuyts.com/self-referential-types-2/) explore the `Move` trait design.
- [Why Pin is a part of trait signatures](https://blog.yoshuawuyts.com/why-pin/) explains the problems with `Pin` that motivate this work.
- [Placing functions](https://blog.yoshuawuyts.com/placing-functions/) proposes syntax for constructing `!Move` types in place.
