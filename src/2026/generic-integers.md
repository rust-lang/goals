# Generic integers

| Metadata         |                                                             |
| :--------------- | ----------------------------------------------------------- |
| Contact          | @clarfonthey                                                |
| Status           | Proposed                                                    |
| What and why     | Allow writing code that's generic over the size of integers |

## Summary

[RFC 3686: Generic Integers V2: It's Time](https://github.com/rust-lang/rfcs/pull/3686) is currently blocked on exploring how exactly this feature would interact with the existing compiler and language, to resolve some of the current unknowns.

In particular, an MVP of this feature would add `u<N>` and `i<N>` const-generic integer types similar to C's `_BitInt(N)` type, with the explicit goal to ensure that these types are literally equivalent to their non-generic versions. This would allow vast simplifications of various APIs and documentation for them, and would be useful even without "filling in" the other values for `N` in the implementation.

## Motivation

### The status quo

Since this goal already has a written RFC, the motivation for this goal is better worded in [the RFC's motivation section](https://github.com/clarfonthey/rust-rfcs/blob/generic-integers-v2/text/0000-generic-integers.md#motivation).

But, in short:

1. Implementations of traits for integer types is particularly painful and requires using macros to output implementations for all relevant integer types.
2. This also results in `rustdoc` output being particular awful, e.g. see [`Shl`](https://doc.rust-lang.org/nightly/std/ops/trait.Shl.html#implementors).
3. And, because this is literally more code, it adds a load of extra work for the compiler to compile, reference, and optimize these implementations, even though a majority of them will not actually be used downstream for any given project.

The full RFC also goes into more detail about other benefits of this implementation long-term, but these are the most relevant examples for this goal.

### What we propose to do about it

This goal would tackle both a language experiment and compiler experiment at the same time. Effectively, the minimum parts we want to have in place are:

* Having the ability to represent `u<N>` and `i<N>` types in the compiler, even if `N` is limited to the current values of 8, 16, 32, 64, and 128.
* Allowing type inference to use the generic parameter of integer types as well, to potentially improve the current fallback on integers to assume `u32` or `i32` if no better option exists.
* Updating the standard library implementation to use these types to improve documentation and code generation.

There are also a few reasonable extensions to this that might become part of the MVP, like:

* Creating wrapper `std::ffi::c_bitint` and `std::ffi::c_bituint` types which replicate the current ABI for these types in C. Unfortunately, the ABI for `_BitInt(128)` and `__int128` *are not* the same on x86_64, which indicates there is value in offering this for Rust as well.
* Offering more values of `N` specifically where `N` is a multiple of 8. This means we would make a decision on how the size and alignment of these types should operate, and how they interface with `repr(packed)`, without determining how stuff like `u0`, `i1`, and `u23` should operate.
* Internally offering a version of `u<N>` and `i<N>` that can also generalize over `usize` and `isize`, since these are distinct types from their same-size counterparts. This would allow further simplifying the standard library's code and potentially be available to end users once ADT const generics are stable.

At least initially, the goal will *not* be to implement the full RFC and some of its stretch goals, namely:

* Non-byte-rounded values of `N` being accepted, and allowing dedicated conversion between e.g. `u1` and `bool`.
* Bitfields, and a "bit-packed" representation for structs.
* Range-based integer types, e.g. `int<0..=100>` for percentages.

### Work items over the next year

| Task                                  | Owner(s) | Notes |
| ------------------------------------- | -------- | ----- |
| Implementation                        | None yet |       |
| Incorporate feedback/changes into RFC | None yet |       |

## Team asks

| Team            | Support level | Notes                                       |
| --------------- | ------------- | ------------------------------------------- |
| [libs]          | Small         | Review code changes                         |
| [compiler]      | Medium        | Review code changes                         |
| [lang]          | Medium        | Review/make decisions on unresolved details |
| [types]         | Medium        | Combination of above                        |

## Funding

Minimum funding would enable the developer for this feature (likely @clarfonthey) to actually spend time on this, since it otherwise would not happen any time soon.

| Purpose     | Cost    | Funded | Sponsor(s) |
| ----------- | ------- | ------ | ---------- |
| Development | Unknown | No     |            |
