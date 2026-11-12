# Specialization: minimal implementation

| Metadata              |                                    |
| :--                   | :--                                |
| Contact      | @jackh726                           |
| Status                | Accepted                           |
| Tracking issue        | [rust-lang/goals#652] |
| Zulip channel         | N/A                                |
| Other tracking issues | [rust-lang/rust#31844]               |
| [lang] champion       | @tmandry                           |
| [types] champion      | @jackh726                          |


## Summary

Work over this year will be done to identify key use cases of specialization and possible designs that could address them, and to implement a minimal, sound design for future stabilization.

## Motivation

### The status quo

Specialization, the ability to branch (potentially through an impl) on type properties known at monomorphization time, is a long-sought feature in Rust. An initial [RFC #1210] and [implementation][tracking] was made that broadly allows overlapping impls as long as one impl is an unambiguous "winner". However, there are soundness issues with this design.

A subset of specialization, coined the [“always applicable”][always applicable] subset, is restricted to implementations that hold regardless of lifetime substitution, roughly corresponding to specializing on concrete types. A feature gate, `min_specialization`, was split out that roughly approximates this subset. The feature was designed for use within the standard library, but was not intended as a stable user-facing feature. There are at the very least implementation bugs, but it is not clear if this subset of specialization is truly sound, or whether or not it can cover all the uses cases people need, both inside the standard library and in user crates.

The next-generation trait solver, set to stabilize, resolves some bugs in the current implementation, though the work remaining is primarily one of design. Additionally, there have been alternative features proposed (`try_as_dyn` and `capability-safe specialization`) that function differently than “always applicable” specialization and cover different – largely overlapping – use cases.

[always applicable]: https://smallcultfollowing.com/babysteps/blog/2018/02/09/maximally-minimal-specialization-always-applicable-impls/#when-is-an-impl-always-applicable
[tracking]: https://github.com/rust-lang/rust/issues/31844

### What we propose to do about it

Given that specialization is largely blocked on design work to address known soundness issues, the bulk of the work within this project goal will be to identify and collect the constraints for a possible stabilizable design (through a survey of existing and expected use cases), document these constraints and propose some design to move towards stabilization (which may only cover some subset of uses), model a stabilizable design, and implement that design within the compiler (possibly adapting existing implementation work).

### Work items over the next year

| Task                         | Owner(s) | Notes  |
| --------------------------   | --------- | ----- |
| Survey use cases             | @jackh726 |       |
| Triage open issues           | @jackh726 |       |
| Prepare design document      | @jackh726 |       |
| Model in a-mir-formality     | @jackh726 |       |
| Implement and iterate design | @jackh726 |       |


## Team asks

| Team       | Support level | Notes |
| ---------- | ------------- | ----- |
| [lang]     | Medium        | Review design document |
| [types]    | Large         | Review design document |
| [libs]     | Small         |       |
| [opsem]    | Small         |       |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | TBD | Full | Google |

## Frequently asked questions
