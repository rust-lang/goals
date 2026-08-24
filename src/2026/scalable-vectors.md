# Sized Hierarchy and Scalable Vectors

| Metadata            |                                    |
| :--                 | :--                                |
| Point of contact    | @davidtwco                         |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#270] |
| Highlight           | Try, never, extern types           |
| [compiler] champion | @davidtwco                         |
| [lang] champion     | @nikomatsakis                      |
| [libs] champion     | @Amanieu                           |
| [types] champion    | @lcnr                              |

## Summary

Over the next year, we will build on the foundational work from 2025 to stabilize
the `Sized` trait hierarchy and continue nightly support for scalable vectors:

- Stabilize the refined `Sized` trait hierarchy (without constness), unblocking extern types
- Propose and implement `const Sized` to support scalable vectors
- Achieve RFC acceptance for [RFC #3838] (Scalable Vectors)
- Land SVE types and intrinsics in stdarch for nightly experimentation
- Continue addressing stabilization blockers for SVE itself
- Begin design work for supporting the Scalable Matrix Extension (SME)

The `const Sized` work (Part II of [RFC #3729]) is deferred to a future goal,
allowing us to deliver value sooner through the trait hierarchy stabilization.
This future work interacts with ongoing [const generics][const-generics-goal]
efforts, as `const Sized` depends on progress in const traits.

[const-generics-goal]: ../2025h2/const-generics.md

## Motivation

Arm has introduced Scalable Vector Extensions (SVE) and Scalable Matrix
Extensions (SME) - powerful SIMD capabilities where vector register width
depends on the CPU implementation rather than being fixed at compile time.
Hardware is generally available, and key Rust stakeholders (Google, Huawei,
Microsoft) have expressed urgent desire for SVE support in Rust.

**The problem:** Scalable vectors don't fit Rust today for three reasons:

1. At the language level, Rust's `Sized`/`?Sized` distinction is too coarse -
   it only distinguishes between types whose "size is known at compile time"
   and whose "size is in metadata." Scalable vectors need a third category:
   types whose size is constant at runtime but unknown at compile time.
2. Unlike fixed-size SIMD types, using scalable vector types require the
   architecture support to be present for even the simplest operations to
   be possible (e.g. returning scalable vectors from functions). At the language
   level, this will necessitate some ability to require the relevant target
   features be present when scalable vectors are used, which is especially
   tricky with trait implementations and generic functions.
3. At the compiler level, we need new infrastructure to generate code for
   scalable vector types and their intrinsics.

**The opportunity:** By extending Rust's type system with a richer `Sized`
hierarchy and adding support for scalable vectors, we can support SVE in AArch64 as well as
support for similar features in other architectures, like RISC-V's "V" Vector Extension; we also
unblock other long-requested features like extern types.

Since SVE [requires a change to the C standard][acle_sve], Rust has an opportunity to be the first
systems programming language with native support for these hardware capabilities.

[acle_sve]: https://github.com/ARM-software/acle/releases/download/r2024Q3/acle-2024Q3.pdf

### The status quo

Significant progress was made in 2025:

- **Sized Hierarchy: Part I** ([rust-lang/rust#137944]) has been merged, introducing
  new non-const sizing traits behind the `sized_hierarchy` feature gate
- **Scalable vector infrastructure** ([rust-lang/rust#143924]) has been merged, adding
  experimental `rustc_scalable_vector(N)` attribute support
- **Hierarchy of Sized traits** ([RFC #3729]) is being implemented experimentally

See the tracking issues for the Sized Hierarchy prerequisite ([rust-lang/rust#144404]) and
for Scalable Vectors themselves ([rust-lang/rust#145052]).

### What we propose to do about it

In 2026, we plan to factor out a subset of [RFC #3729] that can be stabilized
independently: traits like `SizeOfVal` that don't require const trait support.
This subset unblocks extern types ([RFC #1861]), a long-requested feature, while
the const-specific portions needed for SVE itself remain experimental pending
progress on const traits.

Our design axioms:

- **Avoid overfitting.** Extensions to Rust's type system should not be narrowly
  tailored to SVE/SME, but should support similar extensions from other
  architectures and unblock other desired Rust features where practical.
- **Low-level control.** Rust should leverage the full capabilities and
  performance of the underlying hardware features.
- **Rusty-ness.** Extensions should align with Rust's design principles and feel
  like natural extensions of the type system.

The ultimate goal is stable SVE support in Rust, enabling high-performance SIMD
code that works across AArch64 implementations with varying vector register
widths. Following SVE, adding support for Scalable Matrix Extensions (SME) is
the next logical step, enabling efficient matrix processing in Rust.

### Work items over the next year

| Task                                          | Owner(s) or team(s) | Notes                                     |
| --------------------------------------------- | ------------------- | ----------------------------------------- |
| Stabilize Sized trait hierarchy               | @davidtwco          | Unblocks extern types                     |
| Achieve RFC acceptance for [RFC #3838]        | @davidtwco          | Scalable Vectors RFC                      |
| Update and reopen stdarch SVE PR              | @davidtwco          | SVE types and intrinsics                  |
| Address SVE stabilization blockers            | @davidtwco          | Identify and resolve blockers             |
| SME design exploration                        | @davidtwco          | Understand implications for Rust          |

## Team asks

| Team       | Support level | Notes                                                   |
| ---------- | ------------- | ------------------------------------------------------- |
| [compiler] | Medium         | Standard reviews for stabilization and SVE work         |
| [lang]     | Medium        | RFC decision for [RFC #3838], stabilization sign-off    |
| [libs]     | Medium        | Review RFC; review and approve stdarch SVE APIs         |
| [types]    | Medium        | Type System implementation and stabilization sign-off   |

## Frequently asked questions

### What changed from the 2025 goal?

The 2025 goal laid the groundwork: Part I of the Sized Hierarchy was merged,
the core scalable vector infrastructure was merged, and RFC 3729 was accepted.
The 2026 goal pivots to focus on **stabilizing the Sized trait hierarchy** as a
standalone win that unblocks extern types, while SVE continues as a nightly
experiment with stdarch intrinsics.

### Why stabilize the trait hierarchy separately from `const Sized`?

The Sized trait hierarchy provides value
independent of scalable vectors: it unblocks extern types, a long-requested
feature. By stabilizing the non-const portions first, we deliver value sooner.
The `const Sized` work (Part II of RFC 3729) has dependencies on const traits
and is deferred to a future goal.

### How does this unblock extern types?

Extern types need a way to express "this type has no known size, not even at
runtime." The current `?Sized` bound conflates "unsized but has metadata" with
"truly sizeless." The refined trait hierarchy allows distinguishing
these cases, which is the key blocker for extern types.
