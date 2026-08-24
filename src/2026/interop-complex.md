# C interop: `Complex<T>` 

| Metadata              |                                                       |
| :--                   | :--                                                   |
| Point of contact      | @folkertdev                                           |
| Status                | Accepted                                              |
| Other tracking issues | N/A                                                   |
| Zulip channel         | N/A                                                   |
| Funding contact       | [Trifecta Tech Foundation](https://trifectatech.org/) |
| [compiler] champion   | @folkertdev                                           |
| Tracking issue        | [rust-lang/goals#761]                                 |



## Summary

Rust should be able to define an ABI-compatible counterpart to every signature that C can define. This is important for C interop and translation of code from C to rust, using manual FFI, and tooling like `c-bindgen` and `c2rust`. This goal closes a long-standing gap in rust's ability to match C.

## Motivation

### The status quo

Today there are still a number of cases where Rust cannot, in a portable way, express type signatures that C can define.

The C `_Complex` type is one such missing piece. This type is conceptually just 

```rust
#[repr(C)]
struct Complex<T> { 
    real: T, 
    imaginary: T 
}
```

But many ABI pass `Complex<{float}>` and `Complex<{int}>` in a custom way, different from the equivalent C struct. The value of `Complex<T>` as a built-in type is that we can match those ABIs.

```rust
use core::ffi::c_double;

unsafe extern "C" {
    // Complex square root provided by libm.
    safe fn csqrtf(_: Complex<c_float>) -> Complex<c_float>;
}

fn main() {
    let c = Complex::new(-1.0, 0.0);
    assert_eq!(csqrtf(c), Complex::new(0.0, 1.0));
}
```

There is an accepted RFC ([RFC 3892](https://github.com/rust-lang/rfcs/pull/3892)) for `Complex<T>`.  

### What we propose to do about it

We will add `core::num::Complex`, and the callconv implementations to ensure that C `_Complex` and Rust `Complex` are ABI-compatible accross targets. 

There are some ABI differences between Clang and GCC. We've already submitted LLVM PRs for all known issues, and will try to get these merged into LLVM 24:

- [mips](https://github.com/llvm/llvm-project/pull/212119) (already merged)
- [powerpc](https://github.com/llvm/llvm-project/pull/208917)
- [sparc and sparc64](https://github.com/llvm/llvm-project/pull/212340)

This project goal focusses on the internal implementation, and leaves most of the design and implementation of the `std` API for `Complex<T>` as future work. 

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| get LLVM PRs over the finish line | @folkertdev |      |
| implement `core::num::Complex` | @folkertdev  |       |
| hook up intrinsics for `cmul` and `cdiv` | @folkertdev |     | 
| work towards stabilizing the type for FFI purposes (keeping any controversial API unstable) | @folkertdev |
| design (and maybe implement) more of the API (basic operators, methods based on https://en.cppreference.com/c/numeric/complex) | @folkertdev |

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Medium        | This feature touches `callconv` code, which requires careful review |
| [lang]     | Small         | These new types have a new ABI          |
| [libs]     | Small         | Just a vibe check on the names and minimal APIs |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| implementation work | $12,000 | Full | Google |

## Target timeline

The duration of the project is 6 months. Starting from the agreed start date ("Month 1"), the timeline we're targeting is:

- Month 1-3: callconv implementation work
- Month 3-6: work towards stabilization

The expected effort for the work is 1 person-month. 

## Notes

- [#t-compiler > representing &#96;_Complex&#96;](https://rust-lang.zulipchat.com/#narrow/channel/131828-t-compiler/topic/representing.20.60_Complex.60/with/608311781)
- LLVM has `@llvm.experimental.complex.fmul.v2f64` etc. (introduced in https://reviews.llvm.org/D119284)
- LLVM defines builtins like https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/builtins/divdc3.c which we'll need in compiler-builtins

## Frequently asked questions

### How does this goal interact with the [f80, f128 and c_longdouble](https://rust-lang.github.io/rust-project-goals/2026/interop-f80-f128.html) goal

It is possible to have a `Complex<c_longdouble>`, which in some calling conventions needs custom ABI logic. It is a goal to make this combination work across targets. 
