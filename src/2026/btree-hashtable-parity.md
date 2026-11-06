# `BTree`-`HashTable` parity

| Metadata        |                                                               |
| :--             | :--                                                           |
| Contact         | @clarfonthey                                                  |
| Status          | Proposed                                                      |
| What and why    | Make `BTree` types as robust as their `HashTable` equivalents |
| [libs] champion | @clarfonthey                                                  |
| Tracking issue  | [rust-lang/goals#852]                                         |


## Summary

Move the implementation of `BTreeMap` and `BTreeSet` into a separate crate similar to `hashbrown`, and bring their APIs to parity with `HashMap` and `HashSet`. Additionally, work on a "raw" `BTree` API similar to the raw `HashTable` API offered by `hashbrown`.

## Motivation

### The status quo

Right now, the standard library's implementation of `HashMap` and `HashSet` relies on the implementation in the `hashbrown` crate. This has been a very healthy relationship, since it means that the ecosystem can easily suggest changes to ergonomics of the implementation and expose its inner workings without having to deal with the extremely strict stability guarantees of the standard library. There are several ecosystem crates that are able to build upon this work for other primitives like `indexmap` (hash tables with ordered entries) and `dashmap` (concurrent hash tables).

The situation surrounding `hashbrown` is actually [the subject of its own goal](./hashbrown-refactor.md). This goal extends that one by effectively offering the same treatment for the B-tree implementation, proposing to create a new crate for experimentation, to modernize and audit the API and implementation, and to ensure that all these features are at least considered for inclusion in the standard library.

The B-tree implementation has languished due to its complexity and awkwardness, and has led to a lot of anxiety when it comes to working on these types: several features have been "put off" for their implementation due to the complexity of the current code ([custom allocator support](https://github.com/rust-lang/rust/issues/161678), [cursors](https://github.com/rust-lang/rust/issues/107540), etc.) and many downstream users are limited in the ways they can leverage these primitives due to the lack of robust APIs.

Additionally, since `BTreeMap`'s implementation has not been substantially reworked since Rust 1.0, several new Rust features that have improved the ergonomics of working with unsafe code are currently unavailable for `BTreeMap`, which makes the code just that much more annoying to work with.

### What we propose to do about it

We effectively need to give the `BTree` the same treatment we gave `HashMap`, which is to move its code to a separate crate whose code can be iterated upon and improved without the strict stability requirements of the standard library. Additionally, all of the various plans we have for `hashbrown` should be carried over to `BTree`, namely:

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

Right now, it's likely that @clarfonthey will be prioritizing work on `hashbrown` over this implementation, and so, someone else would need to own the implementation. Since the implementation involves a lot of complex memory management and unsafe code, we would be looking for an expert-level Rust developer with both comfort working with unsafe code *and* the experience to ensure that it is sound and correct.

Time estimates are currently vague due to the amount of work that would be needed, and the overlap with the `hashbrown` goal, but it would definitely be on the order of months, not weeks.

| Task           | Experience level              | Time investment |
| -------------- | ----------------------------- | --------------- |
| Implementation | Expert (unsafe B-tree code)   | ??? months      |

## Funding

Minimum funding would enable at least one developer to fully devote themselves to implementing these changes, which otherwise will be put off until work on `hashbrown` is complete before commencing.

| Purpose        | Cost    | Funded | Sponsor(s) |
| -------------- | ------- | ------ | ---------- |
| Implementation | Unknown | No     |            |
