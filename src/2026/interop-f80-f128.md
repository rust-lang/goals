# C interop: `f80`, `f128` and `c_longdouble`

| Metadata              |                                                       |
| :--                   | :--                                                   |
| Point of contact      | @folkertdev                                           |
| Status                | Accepted                                              |
| Tracking issue        | [rust-lang/goals#701]                    |
| Other tracking issues | N/A                                                   |
| Zulip channel         | N/A                                                   |
| Funding contact       | [Trifecta Tech Foundation](https://trifectatech.org/) |


## Summary

Rust should be able to define an ABI-compatible counterpart to every signature that C can define. This is important for C interop and translation of code from C to rust, using manual FFI, and tooling like `c-bindgen` and `c2rust`. This goal closes a long-standing gap in rust's ability to match C.

## Motivation

### The status quo

Today there are still a number of cases where rust cannot, in a portable way, express type signatures that C can define.

The C `long double` type is one such missing piece. Normally rust provides the C-compatible types as type aliases (e.g. `type c_int = i32`), but `c_longdouble` requires additional types: on many platforms it maps to `f64`, but on some it maps instead to an 80-bit or 128-bit float, two types that are not currently available in stable rust.

We will add the types (with no or very limited arithmetic operations) for interop purposes.

### What we propose to do about it

We will add two new storage-only float types to `core::arch` to facilitate with interop:

- `core::arch::x86::x87_f80`, only available on `x86` and `x86_64`
- `core::arch::powerpc::ibm_f128`, only available on `powerpc`, `powerpc64` and `powerpc64le`

The API will be minimal, and will not include any arithmetic operations. Those operations can be added later.

We will also continue to push `f128` forward, both on the LLVM and rustc side.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| implement `core::arch::x86::x87_f80` | @folkertdev  |       |
| implement `core::arch::powerpc::ibm_f128` | @folkertdev  |       |
| `f128` `ToString`/`FromString` | @folkertdev` | There is some active work on these algorithms, we'll try to help out where we can |
| `core::arch::powerpc::ibm_f128` `ToString`/`FromString`  | @folkertdev  |       |
| fix the `f128` windows ABI in LLVM | @folkertdev, @tgross35 | we will likely need support from the LLVM/Microsoft side here |
| stabilize `core::arch::x86::x87_f80` | @folkertdev  |       |
| stabilize `core::arch::powerpc::ibm_f128` | @folkertdev  |       |
| attempt to stabilize `core::ffi::c_longdouble` | @folkertdev  | if the types must have (some) arithmetic operations, we might need more time |

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Medium        | The changes are not complicated, but touch some core types so this might need some design/refactoring to get right |
| [lang]     | Small         | These new types have a new ABI          |
| [libs] | Small             | Just a vibe check on the names and minimal APIs |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| implementation work | $36,000 | Full | Google |

## Target timeline

The duration of the project is 12 months. Starting from the agreed start date ("Month 1"), the timeline we're targeting is:

- Month 1-3: unstable `c_longdouble`, `x87_f80` and `ibm_f128` on nightly
- Month 3-9: improvements to `f128` in rustc and LLVM
- Month 9-12: stabilize `x87_f80` and `ibm_f128`, prepare stabilization of `c_longdouble`

The expected effort for the work is 3 person-months, the long timeline is really due to LLVM release cycles.

## Notes

There is a stalled RFC for some of these types at [rust-lang/rfcs#3456].

Blockers for `f128` support are tracked at [`cfg.has_reliable_f128`](https://github.com/rust-lang/rust/blob/2cfb951a24de2520de67f6911fd1fc0045a2662e/compiler/rustc_codegen_llvm/src/llvm_util.rs#L394-L415).

There is a draft PR [rust-lang/rust#140417] for `c_longdouble` that already lists the right type for `c_longdouble` for many targets.

Further discussion is at [#t-libs > &#96;f80&#96;, &#96;f128&#96; and &#96;c_longdouble&#96;](https://rust-lang.zulipchat.com/#narrow/channel/219381-t-libs/topic/.60f80.60.2C.20.60f128.60.20and.20.60c_longdouble.60/with/604820894).

## Frequently asked questions

(None.)
