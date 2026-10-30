# Relink Don't Rebuild

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Contact | @susitsm                                                                         |
| Status           | Proposed                                                                         |
| What and why     | Massive speedup of rebuilds in large multi-crate projects                                                                |
| Roadmap          | Fast Builds                                                                      |
| Tracking issue   |                                                                                  |
| Other tracking issues | [rust-lang/rust#158844]            |
| Zulip channel    | [t-compiler/relink-dont-rebuild][zulip] |
| [compiler] champion | @petrochenkov |

[zulip]: https://rust-lang.zulipchat.com/#narrow/channel/604410-t-compiler.2Frelink-dont-rebuild

## Summary

Implement the MCP [Relink Don't Rebuild](https://github.com/rust-lang/compiler-team/issues/790), which can give large speedups on [certain edits](https://github.com/rust-lang/cargo/issues/14604#issuecomment-4371471225) when rebuilding large projects with many crates.

There is an existing implementation (#156940) waiting for review. The first goal is to get that through the review process, integrate with cargo, and get it to nightly users.


## Motivation


### The status quo

For a long time, the preferred method for speeding up compilation of large projects was to split them into smaller crates. Today, large projects consist of many crates with a complex dependency graph. When a crate is edited, it causes all dependents to recompile, leading to slow compile times even when the edited crate is small.

### What we propose to do about it

Calculate the public API hash of crates and only recompile dependents when it changes, greatly improving iteration speed for edits not changing the public API of a crate. This is similar to how C compilers only recompile the edited `.c` file then relink the final binary. Other code will not get recompiled unless the header file changes.

The largest concern of adding such public API hash is maintainability: adding new language features should automatically get added to the public API hash, otherwise RDR can quickly become riddled with silent miscompiles. The proposed implementation solves this problem by building on the metadata ("header") files produced by rustc, automatically integrating any new data added to it.

One more concern is the second order effect of implementing RDR without going to the extra mile with how it handles line information. Considering all public item line info changes as public API changes encourages people to write "header style" code. Instead of
```rust
pub fn foo() {
    do_something();
}
```
writing
```rust
pub fn foo() {
    foo_private()
}

fn foo_private() {
    do_something();
}
```
can make the public API more resilient to change. Adding `do_something_else()` to `foo_private` does not change the public API, while adding it to `foo` changes it, which can lead to considerable difference in recompilation speed in a large project, encouraging people to write these "header" functions and make lints for enforcing it. It remains to be discussed whether combating this is desired in the future. There are no plans to do special handling for line info only changes in the first version of RDR.

### Work items over the next year

#### Usable implementation in nightly, with limitations

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Split up the existing implementation into smaller PRs, review | @susitsm, @petrochenkov  | 2 months |
| Cargo integration | @susitsm  | 1 month |

#### Prepare for stabilization

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Restore compiler diagnostics | @susitsm | 1 month |
| Optimize the implementation | @susitsm | 2 months |
| Integrate with rustdoc | @susitsm | 1 month |

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    | Small         | review cargo integration                 |
| [compiler] | Medium        | dedicated reviewer                             |

## Funding

The duration of the project is 7 months. The two subgoals can be funded independently.

- Month 1-3 (Usable implementation in nightly)
  - This stage could already provide benefits for nightly users, at the cost of possibly incorrect line info in diagnostics and debuginfo of dependents. This would greatly help `cargo check` runs in IDE-s, since debuginfo is not a concern there.
  - Some diagnostics will not be available
  - Lot of the work is already done, needs review.
- Month 4-7 (Prepare for stabilization)
  - Performance optimizations
  - Rustdoc integration
  - Restoring diagnostics

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor - Nightly implementation | $30,000 | No | |
| Contributor - Prepare for stabilization | TBD | No | |


## Frequently asked questions
