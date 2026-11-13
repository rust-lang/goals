# Stabilize never type (`!`)

| Metadata                |                                                |
| :--                     | :--                                            |
| Contact        | @WaffleLapkin                                  |
| Status                  | Completed                                      |
| Tracking issue          | [rust-lang/goals#653]             |
| Other tracking issues   | [rust-lang/rust#35121]                         |
| Highlight               | Try, never, extern types                       |
| Zulip channel           | N/A                                            |


## Summary

Stabilize the never type aka `!`.

## Motivation

### The status quo

The never type has been unstable for **10** years, with all previous attempts to stabilize it failing.
There is a plan to stabilize it, however it still needs to be implemented in reality. 

### What we propose to do about it

Implement the plan!
There are only a few outstanding issues to be solved.
After they are done, the never type can be stabilized.

### Work items over the next year

| Task                                                                 | Owner(s)                      | Notes                                           |
| -------------------------------------------------------------------- | ----------------------------- | ----------------------------------------------- |
| Don't consider `Result<T, !>` as `must_use` unless `T` is            | @WaffleLapkin                 | [rust-lang/rust#148214]                      |
| Improve dead-code lint to work with never type fallback              | @jdonszelmann, @WaffleLapkin  | [rust-lang/rust#146085]                         |
| Further restrict what coercions are allowed on places of type `!`    | @WaffleLapkin                 | [rust-lang/rust#131297]                         |
| Re-assess the breakage needed for the fallback change              v | @WaffleLapkin                 |                                                 |
| Stabilize the never type!                                            | @WaffleLapkin                 |                                                 |

## Team asks

| Team       | Support level | Notes                                                                          |
| ---------- | ------------- | ------------------------------------------------------------------------------ |
| [types]    | small         | We expect to only need normal reviews                                          |
| [lang]     | small         | Most of the plans / design was already approved, only minor sign-offs required |

## Frequently asked questions

N/A
