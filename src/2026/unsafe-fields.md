# Stabilize Unsafe Fields

| Metadata           |                                                                                           |
| :--                | :--                                                                                       |
| Contact   | @jswrenn                                                                                  |
| [lang] champion    | @nikomatsakis                                                                             |
| Status             | Accepted                                                                                  |
| Tracking issue     | [rust-lang/goals#273]                                                        |
| Zulip channel      | https://rust-lang.zulipchat.com/#narrow/channel/213817-t-lang/topic/unsafe.20fields.20RFC |

## Summary

Complete and stabilize field safety tooling ([RFC3458](rust-lang/rfcs/3458)).

## Motivation

The absence of a mechanism for denoting the presence of library safety invariants increases both the risk of working with `unsafe` code and the difficulty of evaluating its soundness.

### The status quo

Presently, Rust lacks mechanisms for denoting when fields carry library safety invariants, and for enforcing extra care around their use. Consequently, to evaluate the soundness of `unsafe` code (i.e., code which relies on safety invariants being upheld), it is not enough to check the contents of `unsafe` blocks — one must check all places (including safe contexts) in which safety invariants might be violated. (See [*The Scope of Unsafe*](https://www.ralfj.de/blog/2016/01/09/the-scope-of-unsafe.html))

For example, consider this idealized `Vec`:

```rust
pub struct Vec<T> {
    data: Box<[MaybeUninit<T>]>,
    len: usize,
}
```

Although `len` is bound by a safety invariant, it is trivial to violate its invariant in entirely safe code:

```rust
impl Vec<T> {
    pub fn evil(&mut self) {
        self.len += 2;
    }
}
```

Rust cannot enforce that modifications of `len` require `unsafe`, because the language does not provide the programmer a way of communicating to the compiler that `len` carries safety invariants.

After more than a decade of [discussion](rust-lang/rfcs/381), an [RFC for field safety tooling](rust-lang/rfcs/3458) has been accepted and a preliminary implementation is available with `#![feature(unsafe_fields)]`. Instability and gaps in supporting tooling (i.e., clippy, rustdoc, and rustfmt) prevent this feature from being utilized widely.

### The "shiny future" we are working towards

Rust programmers will use the `unsafe` keyword to denote fields that carry library safety invariants; e.g.:

```rust
struct Vec<T> {
    // SAFETY: The elements `data[i]` for
    // `i < len` are in a valid state.
    unsafe data: Box<[MaybeUninit<T>]>,
    unsafe len: usize,
}
```

Rust will require that usages of `unsafe` fields which could violate their safety invariants must *only* occur within `unsafe` contexts.

### Work items over the next year

Over the next year, we will complete tooling support, documentation, and stabilization of Unsafe Fields according to the steps documented in [*Tracking issue for RFC 3458: Unsafe fields*](rust-lang/rust/132922):

| Task                                                       | Owner(s) | Notes |
| ---------------------------------------------------------- | -------- | ----- |
| Implement clippy support.                                  | @jswrenn |       |
| Implement rustdoc support.                                 | @jswrenn |       |
| Implement rustfmt support.                                 | @jswrenn |       |
| Add Book documentation.                                    | @jswrenn |       |
| Add Standard Library documentation.                        | @jswrenn |       |
| Add Reference documentation.                               | @jswrenn |       |
| Add Style Guide documentation.                             | @jswrenn |       |
| Write Stabilization Report.                                | @jswrenn |       |
| Stabilize                                                  | @jswrenn |       |

## Team asks

| Team       | Support level | Notes                                           |
| ---------- | ------------- | ----------------------------------------------- |
| [book]     | Small         | Will need approval for book changes.            |
| [clippy]   | Small         | Will need approval for clippy support.          |
| [lang]     | Small         | Will need approval for stabilization.           |
| [libs]     | Small         | Will need approval for documentation changes.   |
| [spec]     | Small         | Will need approval for reference changes.       |
| [style]    | Small         | Will need approval for style guide changes.     |
| [rustdoc]  | Small         | Will need approval for rustdoc support.         |
| [rustfmt]  | Small         | Will need approval for rustfmt support.         |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Rustdoc support for unsafe fields | Ask | Full | Google |
