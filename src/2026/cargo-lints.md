# Stabilize Cargo's linting system

| Metadata         |                                                                                  |
| :--              | :--                                                                              |
| Contact | @epage                                                                           |
| Status           | Completed                                                                        |
| Tracking issue   | [rust-lang/goals#650]                                               |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |


## Summary

Polish the unstable linting system in Cargo so users can get the benefit of the current and future lints for their workspace and packages.

## Motivation

### The status quo

To avoid annoying users and making them numb to diagnostics,
The Cargo Team has limited Cargo's warnings to only those that have some way of being silenced.
For instance, users can easily end up with an old `workspace.resolver`.
This field is normally defaulted based on the edition but that doesn't apply for virtual workspaces.
To help users in a way that can still be silenced, Cargo warns if `workspace.resolver` is unset and if any workspace member is on Edition 2021 or later.
Users can silence this by setting `workspace.resoler`.
However, we can't raise awareness of `workspace.resolver` being stale because there wouldn't be a way to silence it.

When considering a linting system in [rust-lang/cargo#12235]
we collected a good number of issues that would benefit from having one.

### What we propose to do about it

This comes in two parts ([rust-lang/cargo#12235]):
- Polish the linting system to be up to the quality of rustc and clippy
- Implement an initial batch of lints to serve as examples for future lints, vet our design, and provide a motivation for stabilization and use

In particular, we feel that native support for detecting unused dependencies
([rust-lang/cargo#15813])
would provide a strong motivation for use of the linting system because it will,
over time, improve build times.

See [the Cargo docs](https://doc.rust-lang.org/nightly/cargo/reference/lints.html) for a list of implemented lints.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Unused dependencies lint | *epage*  |       |
| Integrate with warning, error reporting and control | *epage*  |       |
| Document lint contribution process | *epage*  |       |
| Misc polish | *epage*  |       |

See [rust-lang/cargo#12235] for more details

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    | Small         | Code reviews and maybe a design discussion or two |
| [compiler] | Small         | Review our initial batch of lints to ensure they provide an example of adapting the existing lint guidelines to Cargo |
| [clippy] | Small           | Review our initial batch of lints to ensure they provide an example of adapting the existing lint guidelines to Cargo |

## Frequently asked questions
