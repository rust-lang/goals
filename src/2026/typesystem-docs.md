# Type System Documentation

| Metadata              |                                    |
|:----------------------|:-----------------------------------|
| Contact      | @BoxyUwU                           |
| Funding contact | [Hexcat](https://hexcat.nl/) |
| Status                | Accepted                           |
| Tracking issue        | [rust-lang/goals#405] |
| Other tracking issues | [rustc-dev-guide#2663]             |
| Zulip channel         | [#t-compiler/rustc-dev-guide]                            |
| [types] champion      | @boxyuwu                           |

[#t-compiler/rustc-dev-guide]: https://rust-lang.zulipchat.com/#narrow/channel/196385-t-compiler.2Frustc-dev-guide/topic/Type.20System.20Docs.20Rewrite/with/562996633

## Summary

Improve documentation of type system components to aid in types team onboarding and communication about changes to the type system.

## Motivation

### The status quo

The type system is a very complex and critical component of the compiler. It is currently lacking in documentation, and the documentation that *does* exist is often inadequate for gaining a thorough understanding of a given part of the type system (or simply outdated as it was written many years ago).

The lack of documentation makes onboarding difficult for new contributors and require a lot of energy from experienced contributors who are now responsible for explaining everything from scratch themselves. A similar problem also occurs when reviewing changes to the type system, as there is no documentation it can be difficult to bring everything back into cache and be confident that the subtleties of the area being changed have all been taken into account.

### What we propose to do about it

In the previous period, @BoxyUwU and @lcnr put together a list of type system topics that should be covered: [Type System Documentation Overhaul rustc-dev-guide#2663](https://github.com/rust-lang/rustc-dev-guide/issues/2663).

The next steps are to pick items from this list, check them against the [rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/) and document what's missing or outdated. We already have some concrete ideas for which items to start working on.

Ultimately, all type system components should be thoroughly documented. Contributors should not find themselves in a position where knowledge of the type system is *only* attainable by speaking with types team members instead of having readily available documentation to read.


### Work items over the next year


| Task                                  | Owner(s) | Notes |
|---------------------------------------|----------|-------|
| Const Generics Docs | @BoxyUwU |       |
| High Level Trait Solving Architecture Docs | @BoxyUwU |       |
| New Solver Docs | @BoxyUwU @lcnr |       |
| Item/Non-Item WF-Check Docs | @BoxyUwU |       |


## Team asks

| Team    | Support level | Notes                        |
|---------|---------------|------------------------------|
| [types] | Small         | Discussion and moral support |
