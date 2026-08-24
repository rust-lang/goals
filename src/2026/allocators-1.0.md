# Allocators 1.0

| Metadata         |                                                           |
| :--              | :--                                                       |
| Point of contact | @nia-e                                                    |
| Status           | Accepted                                                  |
| What and why     | Enabling use of custom allocators in the standard library |
| Timespan         | 2026-2027                                                 |
| Zulip channel    | [#t-libs/wg-allocators][channel]                          |
| [libs] champion  | @nia-e                                                    |
| [lang] champion  | @joshtriplett                                             |
| Funding contact  | [Hexcat](https://hexcat.nl/)                              |
| Tracking issue   | [rust-lang/goals#762]                                     |


[channel]: https://rust-lang.zulipchat.com/#narrow/channel/197181-t-libs.2Fwg-allocators

## Summary

Work on expanding the standard library to natively integrate with custom allocators, while facilitating key ecosystem usecases for implementors and consumers.

## Motivation

### The status quo

Custom allocators in the standard library have been under construction in some capacity at least since [RFC 1398][alloc-rfc] in 2015, with the nightly [tracking issue on the matter][alloc-issue] open since 2016. Much work was delayed by API and usecase uncertainty, while language and typesystem support slowly pushed forward to enable previously untenable usecases.

As of the time of proposing, work is restarting in the area and there is momentum to push for deeper library integration.

[alloc-rfc]: https://github.com/rust-lang/rfcs/pull/1398/changes
[alloc-issue]: https://github.com/rust-lang/rust/issues/32838

### What we propose to do about it

Reconstitute the allocator working group, using it as a liaison between prospective implementors and the library team. Then, stabilising a few core standard library APIs - notably the base `Allocator` trait - will enable significant experimentation both in the ecosystem and in the standard library.

Much of this will consist of relatively straightforward implementation and review work; however, past experience shows that allocators are a particularly subtle source of soundness and correctness issues, thus deserving of significant scrutiny. Work going forward will focus on offering as robust and expressive a core as possible, slowly extending safe and stable APIs into more of the standard library alongside language tooling to make working with custom allocators simple.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Base trait stabilization | @nia-e | |
| Fallible collections | @nia-e | |
| Deallocators and other trait extensions | @nia-e | |
| Deeper language integration | @nia-e | @nia-e, @joshtriplett |

Following the stabilization of the base `Allocator` trait, most mentioned work can begin happening in parallel. The reconstituted allocator working group will be leading this effort.

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [libs]     | Large  | |
| [lang]     | Medium | Reviewing trait evolution proposals |
| [compiler] | Small  | Coupling allocators to backend intrinsics |
| [types]    | Small  | |


## Funding

Allocators are currently being worked on mostly on an ad-hoc basis, as time permits. The biggest bottleneck remains review and testing time.

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Implementation, testing, and review | Ask | Partial | |

## Frequently asked questions

None yet :D
