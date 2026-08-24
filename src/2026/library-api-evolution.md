# Evolving the standard library API across editions

| Metadata            |                                    |
| :--                 | :--                                |
| Point of contact    | @Amanieu                           |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#633] |
| Zulip channel       | N/A                                |
| [libs] champion     | @Amanieu                           |
| [edition] champion  | @ehuss                             |
| [compiler] champion | @yaahc                             |
| [rustdoc] champion  | @GuillaumeGomez                    |


## Summary

Add a mechanism for edition-dependent re-exports which allows the standard library to make larger API changes across editions.

## Motivation

The library team is generally very careful about stabilizing new APIs in the standard library because once such an API is stable, it must remain unchanged forever. This is necessary to maintain Rust's [stability guarantees](https://rustc-dev-guide.rust-lang.org/stability-guarantees.html) which ensure that rustc is able to compile Rust code from 2015 and that new crates are able to interoperate with old crates.

However there exist several stable standard library APIs which we would wish to change to improve ergonomics and safety, if it were possible. While the library API team is not committed to making any specific change, several changes have previously been discussed:

- [Improving the `Command` builder.](https://github.com/rust-lang/rust/issues/145957)
- Changing the `write!` macro to require a single consistent trait for the output sink.
- [Changing `Mutex` to be non-poisoning by default.](https://github.com/rust-lang/rust/issues/149359)
- Improving the `Path` API.
- Changing the semantics of `clamp`/`min`/`max` to improve performance on common platforms.

The exact changes are outside the scope of this project goal and should be discussed elsewhere. This goal is only about giving us the tools needed to potentially make such changes.

### The status quo

Currently the only existing tool for making such changes is deprecating the old API and introducing a new one with a similar name. For example, this is what was done with `Command::pre_exec` and `Command::before_exec`. However this approach has several downsides:

- It can often be hard to pick a good alternate name for a function or type since the obvious names will have already been chosen for the original (stable) API. Consider the case of `Atomic*::fetch_update` which is being deprecated in favor of `Atomic*::update` and `Atomic*::try_update`: the original name was arguably better since it clarifies what value is returned.
- A deprecation is a strong signal that effectively forces every still-maintained crate to update to the new API. This can cause a lot of ecosystem churn as everyone updates to the new API.
- Deprecated types in the public API of a crate are particularly problematic because migrating to the new API would result in a breaking change which requires a major version bump.

### What we propose to do about it

We would like the ability to make larger API breaking changes across editions. To do so we would like to add a mechanism for _edition-dependent re-exports_. This would allow a single path in the standard library to resolve to different items depending on the edition of the code that is referencing it. For example:

```rust
// std
mod foo {
    struct Bar2024;
    struct Bar2027;

    // Exact syntax for this is not definitive
    #[edition = "2024"]
    use Bar = Bar2024;
    #[edition = "2027"]
    use Bar = Bar2027;
}

// 2024 edition crate
use std::foo::Bar; // resolves to std::foo::Bar2024

// 2027 edition crate
use std::foo::Bar; // resolves to std::foo::Bar2027
```

This would allow for major changes in the standard library API while still allowing for backwards compatibility with older editions since the items used by older editions are still available, just through a different path. Additionally, automatic edition upgrade could automatically re-write paths to use the one for the old item, which ensures that code behavior doesn't change during the automatic upgrade.

### Work items over the next year

| Task                       | Owner(s) | Notes                                          |
| -------------------------- | -------- | ---------------------------------------------- |
| Draft RFC                  | @Amanieu |                                                |
| Implement compiler support | @Amanieu | Needs a reviewer familiar with name resolution |

## Team asks

| Team       | Support level | Notes                                                                        |
| ---------- | ------------- | ---------------------------------------------------------------------------- |
| [compiler] | Medium        | Design discussions and implementation review.                                |
| [lang]     | Small         | Review of the feature and lang implications.                                 |
| [libs]     | Large         | Determine what API changes should be made across editions.                   |
| [edition]  | Large         | Review the feasibility of this proposal as well as the specific API changes. |
| [rustdoc]  | Medium        | Figure out how such API changes should be presented in the API docs.         |
| [types]    | Small         | Review of any changes to HIR ty lowering or method resolution                |
