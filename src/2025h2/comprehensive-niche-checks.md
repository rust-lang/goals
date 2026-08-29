# Comprehensive niche checks for Rust

| Metadata           |                                                                                                                                                                      |
| :------------------| :--------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Contact   | @1c3t3a                                                                                                                                 |
| Status             | Proposed                                                                                                                                                             |
| Tracking issue     | [rust-lang/goals#262]                                                                                                                                   |
| Zulip channel      | https://rust-lang.zulipchat.com/#narrow/channel/435869-goals/topic/Null.20and.20enum-discriminant.20runtime.20checks.20in.20.28goals.23262.29/with/508256920 |
| [compiler] champion | @saethlin |
| [opsem] champion | @saethlin |

## Summary

Add runtime checks to rustc that check for valid niche values. This is an
extension of a [previous project goal](https://goals.rust-lang.org/2025h1/null-enum-discriminant-debug-checks.html)
that added null pointer and enum checks and generally works towards checking for
Undefined behavior at runtime (in debug builds / behind compiler flags).

The check should fire for invalid values that are already created by inserting
a check at load-time / on function entry.

## Motivation

Rust has a few types that specify "invalid values" for themselves, which means
values that can never be occupied by an instance of this type. An example of
that would be `0` for `std::num::NonZeroUsize`, or `null` for
`std::ptr::NonNull`. These so called *niches* are used for optimizations, for
example when being used together with `Option`, where `Option<NonNull<T>>` has
the same size as `*mut T`.

In safe Rust it is never possible to construct such values, but that is
different for unsafe Rust or FFI. E.g. with Rusts growing interop with C (e.g.
in the Linux Kernel) and C++ it makes sense to expose such types over the
language boundary. That can make for situations where a valid C or C++ value
becomes an invalid Rust value.

### The status quo

While [Miri](https://github.com/rust-lang/miri) exists and does a great job at
catching various types of UB in unsafe Rust code, it has the downside of only
working on pure Rust code. Extern functions can not be called and a mixed
language binary is unable to be executed in Miri.

We already added support for checking enum discriminants which is somewhat related,
but this does not cover general niches.

Selecting where to insert these runtime checks for UB can be tricky. For example,
if we want to detect a call in C++ which passes a null pointer to a Rust function
that takes a reference (which thus gets the LLVM `nonnull` attribute), we cannot
just insert a check inside the Rust function, because our check would be trivially
optimized out (the parameter is guaranteed to not be null!). At a minimum, codegen
needs to also not use such parameter attributes when niche checks are enabled
then also insert the checks in a new block at the start of the function before the
parameter is used.

In addition, technically any typed copy asserts the validity of the copied value
as the type in question. But naively inserting a runtime check before every typed
copy of a value with a niche can more than double compile time:
https://github.com/rust-lang/rust/pull/104862#issuecomment-1773363404.

Thus validating all niches will require something more strategic, such as
only inserting checks when SSA values are loaded from memory.

### The next 6 months

Within the next six months we would like to do the following things:

1. Develop a comprehensive niche value check for all niches.
2. Insert it ideally when values are loaded from memory (e.g. a "checked-load") or on function entry.
3. Migrate the enum check to the same system mentioned in 2.

### The "shiny future" we are working towards

Similar to how [UBSan](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html)
exists in Clang, we would like to see an option in Rust to detect undefined
behavior at runtime. As mentioned above, this is critical for cross-language
interoperability and can help catch UB before it reaches production.

The extension of these checks can be done step-by-step, keeping in mind the
overhead. Different checks can be enabled by separate flags (like in UBSan).
Eventually we would like to check (sanitize) most items listed as
[UB in Rust](https://doc.rust-lang.org/reference/behavior-considered-undefined.html).

## Ownership and team asks

**Owner:** [@1c3t3a](https://github.com/1c3t3a), [@jakos-sec](https://github.com/jakos-sec)

| Task                         | Owner(s) or team(s)                                                                | Notes                                    |
|------------------------------|------------------------------------------------------------------------------------|------------------------------------------|
| Discussion and moral support | ![Team][] [compiler], [opsem]                                                     |                                          |
| Implementation               | @1c3t3a], @jakos-sec  |                                          |
| MCP decision                 | ![Team][] [compiler]                                                              | Where to insert the check / checked load |
| Dedicated reviewer           | ![Team][] [compiler], [opsem]                                                      | @saethlin                                |

### Definitions

For definitions for terms used above, see the [About > Team Asks](https://goals.rust-lang.org/about/team_asks.html) page.

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

## Frequently asked questions

None yet.
