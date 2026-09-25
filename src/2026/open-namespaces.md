# Implement Open Rust Namespace Support

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Contact | @epage                                                                           |
| Status           | Accepted                                                                         |
| Tracking issue   | [rust-lang/goals#256]                                               |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |

## Summary

Navigate the cross-team design work to get [RFC #3243] implemented.

**Needs contributor:** This goal needs contributors to help complete the implementations in Cargo, rustc, and crates.io. The work spans multiple repositories and involves cross-team coordination.

## Motivation

[RFC #3243] proposed opening up namespaces in Rust to extension,
managed by the package name with crates-io putting access control on who can publish to a crate's API namespace.
This covers multiple teams and needs a lot of coordination to balance the needs of each team as shown on the [rustc tracking issue](https://github.com/rust-lang/rust/issues/122349).

### The status quo

- Cargo support is partially implemented.
- Compiler design is agreed on and partially implemented.
- There is a crates-io prototype for a previous iteration of RFC 3243 but that code base has likely diverged a lot since then.

### The next 6 months

- Implement Cargo and compiler support for this to be experimented with and allow crates-io work.
- Understand what changes are needed for crates.io support, and what it will take to implement these.

### The "shiny future" we are working towards

### Work items over the next year

| Task                     | Owner(s) | Notes |
| ------------------------ | -------- | ----- |
| Implement the RFC (rustc) | @TaKO8Ki |       |

## Team asks

| Team       | Support level | Notes                         |
| ---------- | ------------- | ----------------------------- |
| [cargo]    | Small         |                               |
| [compiler] | Small         | Design discussions, PR review |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | TBD | No | |

## Help wanted

| Task                       | Experience level | Time investment |
| -------------------------- | ---------------- | --------------- |
| Implement the RFC (Cargo)     | TBD              | TBD             |
| Implement the RFC (rustc)     | TBD              | TBD             |
| Implement the RFC (crates.io) | TBD              | TBD             |

## Frequently asked questions
