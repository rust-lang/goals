# Stabilizing `f16`

| Metadata              |                                                       |
| :--                   | :--                                                   |
| Point of contact      | @folkertdev                                           |
| Status                | Accepted                                              |
| Tracking issue        | [rust-lang/goals#655]                    |
| Other tracking issues | [rust-lang/rust#116909]                               |
| Zulip channel         | N/A                                                   |
| Funding contact       | [Trifecta Tech Foundation](https://trifectatech.org/) |
| [compiler] champion   | @tgross35                                             |

## Summary

In recent years we've seen increasing hardware support for the `f16` float type. Support was originally motivated by machine learning/AI, but `f16` has since also found applications in other domains like graphics and physics simulations.

With LLVM 22, the remaining blockers in the backends have been cleared for `f16`, and therefore stabilizing this type in 2026 is realistic.

## Motivation

### The status quo

The `f16` is unstable. The implementation in the compiler is mostly complete, with some missing const support. For `f16` LLVM 22 has the support we need.

### What we propose to do about it

We will stabilize `f16`. There is not much to design at this point.

When rustc drops support for LLVM 21 (summer 2026) we can clean up the implementation significantly.

The LLVM backend provides implementations for some `f16` functions (type conversions, arithmetic, etc.)
that are not available when using the cranelift backend. We propose to add implementations to
`rust-lang/compiler-builtins` as needed to get the cranelift backend on-par with the LLVM backend.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| complete support in const-eval/miri | @folkertdev  |       |
| improve support in `rustc_codegen_cranelift` | @folkertdev, @bjorn3, @tgross35  |       |
| remove `cfg(target_has_reliable_f16)` | @folkertdev, @tgross35 | |
| write the stabilization report | @folkertdev, @tgross35  |       |

## Team asks

With @tgross35 as the dedicated reviewer, the asks of the teams are limited.

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | small              |                                         |
| [libs]     | small              |                                         |
| [lang]     | small              | occasionally being fast-tracked would be nice |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | $25,000 | Full | [VectorWare](https://vectorware.com/), [OpenAI](https://openai.com/) |

## Target timeline

The duration of the project is 6 months. Starting from the agreed start date ("Month 1"), the timeline we're targeting is:

- Month 1-3: finish the implementation work items
- Month 3-4: publish stabilization report
- Month 6: open stabilization PR

The expected effort for the work is 2 person-months.

## Notes

An earlier version of this project goal also mentioned `f128`.
We decided to split this out because it is much further away from stabilization,
and is related to interop as well (via C `long double` on some targets).

## Frequently asked questions

### What do I do with this space?
