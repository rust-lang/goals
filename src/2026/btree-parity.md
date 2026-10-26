# `BTree` feature parity

| Metadata         |                                                               |
| :--------------- | ------------------------------------------------------------- |
| Contact          | @clarfonthey                                                  |
| Status           | Proposed                                                      |
| What and why     | Make `BTree` types as robust as their `HashTable` equivalents |

## Summary

Right now, the standard library has the benefit of the `hashbrown` crate: a modern hash table implementation that can be reused by the ecosystem for more complicated situations, which is explicitly used for the libstd `HashMap` and `HashSet` types.

A very related set of types, `BTreeMap` and `BTreeSet`, currently have gotten so overbearing in the standard library that they have a bit of a reputation of being hard to deal with. We should fix this, both factoring the implementation out into its own crate where the API can be experimented more easily, and to resolve concerns on custom allocators, cursors, and other desired APIs.

## Motivation

### The status quo

Right now, the standard library's implementation of `HashMap` and `HashSet` relies on the implementation in the `hashbrown` crate. This has been a very healthy relationship, since it means that the ecosystem can easily suggest changes to ergonomics of the implementation and expose its inner workings without having to deal with the extremely strict stability guarantees of the standard library. There are several ecosystem crates that are able to build upon this work for other primitives like `indexmap` (hash tables with ordered entries) and `dashmap` (concurrent hash tables).

In the standard library, we also have the types `BTreeMap` and `BTreeSet` for an analogous data structure, however, these types have languished a lot because they don't have the benefit of being an external crate like `hashbrown`. This has lead to a lot of anxiety when it comes to working on these types: several features have been "put off" for their implementation due to the complexity of the current code (custom allocator support, cursors, etc.) and many downstream users are limited in the ways they can leverage these primitives due to the lack of robust APIs.

Additionally, since `BTreeMap`'s implementation has not been substantially reworked since Rust 1.0, several new Rust features that have improved the ergonomics of working with unsafe code are currently unavailable for `BTreeMap`, which makes the code just that much more annoying to work with.

### What we propose to do about it

We effectively need to give `BTreeMap` the same treatment we gave `HashMap`, which is to move its code to a separate crate whose code can be iterated upon and improved without the strict stability requirements of the standard library. Additionally, all of the various plans we have for `hashbrown` should be carried over to `BTree`, namely:

* This external crate should offer a generic `BTree` type which offers direct access to the underlying B-tree for users to create their own primitives on top of it. (for example, using ranges for keys).
* All unstable APIs should be implemented and offered on the crate for the ecosystem to use before being stabilized in the standard library. Any APIs which are "not ready yet" can be easily tested in the ecosystem before being added to the standard library.
* Support for custom allocators should be properly implemented in a way that can be stabilized.
* The unsafe code in `BTree` should be modernized and documented.

### Work items over the next year

| Task                                               | Owner(s) | Notes                         |
| -------------------------------------------------- | -------- | ----------------------------- |
| Move `BTreeMap`/`Set` implementations to new crate | None yet |                               |
| Raw `BTree` implementation/API                     | None yet |                               |
| Custom allocator support                           | None yet | Currently exists, but brittle |
| Unsafe code auditing/refactoring                   | None yet |                               |

## Team asks

| Team        | Support level | Notes                              |
| ----------- | ------------- | ---------------------------------- |
| [libs]      | Large         | Review API proposals, code changes |
| [infra]     | Small         | New repository and CI setup        |
| [bootstrap] | Small         | Subtree for new crate              |

## Help wanted

| Task           | Experience level              | Time investment |
| -------------- | ----------------------------- | --------------- |
| Implementation | Advanced (unsafe B-tree code) | ??? months      |

## Funding

| Purpose        | Cost    | Funded | Sponsor(s) |
| -------------- | ------- | ------ | ---------- |
| Implementation | Unknown | No     |            |
