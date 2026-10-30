# `hashbrown` Refactor

| Metadata         |                                               |
| :--------------- | --------------------------------------------- |
| Contact          | @clarfonthey                                  |
| Status           | Proposed                                      |
| What and why     | Refactor `hashbrown` and move it into `alloc` |

## Summary

The `hashbrown` crate should be modernized and have most of its API surface moved to the standard library under the `alloc` crate.

## Motivation

### The status quo

Right now, the standard library's implementation of `HashMap` and `HashSet` rely on the implementation in the `hashbrown` crate. This crate contains both the hash table implementation and the concrete uses of `Map` and `Set`, but it also has a lot of APIs that have not been moved into the standard library which provide lots of real use cases like `entry_ref`, which allows using the `Entry` API without eagerly promoting types to their owned version.

The implementation for hashbrown is also very old, designed before the standard library had correctly described its semantics for `MaybeUninit`, without any safety comments for tracking the invariants held by its unsafe code.

Today, many external crates are forced to rely on a `hashbrown` crate dependency instead of standard library types, despite the fact that this *is* the implementation for the standard library. Some prominent examples are the `indexmap` and `dashmap` crates, which build upon the functionality in `hashbrown` to provide ordering and concurrency guarantees on top of the ordinary hash tables. These crates could be instead depending on only the standard library, but currently cannot.

Additionally, the `hashbrown` crate being a direct dependency of the standard library while also depending on those same standard library internals means that the hash table types cannot be in the `alloc` crate, which means that external crates have to depend on it for `no_std` use cases.

### What we propose to do about it

Simply put, the `hashbrown` crate needs a lot of work done to refactor its internals and audit unsafe code, with the end goal of effectively all of its API surface being moved into the standard library *and* ecosystem crates being able to start depending on the standard library instead of `hashbrown`.

Besides the general desire to give `hashbrown` some love, there are a few concrete criteria that are desired:

* The `hashbrown` implementation should be refactored so that there is a clean implementation hierarchy between `HashTable` being used by `HashMap` being used by `HashSet`. Currently, all three types have to make use of an internal `RawTable` type and depend on its internals. "Dogfooding" the raw table implementation and updating it with missing APIs is a good way to ensure it's robust enough for the ecosystem.
* The `RawTable` internals for `hashbrown` should be effectively replaced entirely with high-level `HashTable` APIs. This will in itself get rid of a lot of the weird invariants that the raw tables have to uphold, ensuring that the code can more easily be audited for correctness.
* The `HashTable` type should be added to the standard library as an API that can be used by external crates.
* All API surface in `hashbrown` should be at least audited for inclusion in the standard library, regardless of whether they're actually added or not.
* `HashTable`, `HashMap`, and `HashSet` should also have their allocator support stabilized in the standard library.
* Ideally, downstream users should be consulted to ensure that these APIs are sufficient for their needs. While many crates will not be immediately using the standard library types due to MSRV concerns, we should put the API in a place where, once MSRV is bumped, they can start using them.

### The "shiny future" we are working towards

There are lots of other things we could learn from `hashbrown`, and there is in particular a way of doing more to support the ecosystem's use of hash tables:

* Making a decision on the inclusion of `Equivalent` and `Comparable` traits in the standard library. A lot of users specifically prefer the use of this API for `hashbrown`'s hash tables, but no extensive audit has been done on whether this is the right design for the standard library, or whether new traits are necessary.
* Collecting/implementing more extensive benchmarks for `hashbrown`'s implementation. Currently, a lot of potential performance improvements have been suggested for hashbrown, but the lack of extensive benchmarking makes it unclear which, if any, would have a positive effect in practice.
* Offering one or more new `Hasher` implementations to the standard library. Right now, `hashbrown` uses a non-HashDOS hasher by default because a number of users have stated that their use cases are too performance-sensitive to use the default hasher provided by the standard library. The compiler also uses its own hasher for this reason. The standard library should consider offering at least one "better-performing" hasher, or maybe even more.
* Providing stronger guarantees of `Hash` implementations in the standard library so users can rely on their stability. Currently, the compiler needs to implement its own `StableHash` trait to combat this.

### Work items over the next year

| Task                                              | Owner(s) | Notes                       |
| ------------------------------------------------- | -------- | --------------------------- |
| Make `Set` rely entirely on `Map`                 | None yet | Will involve new APIs       |
| Make `Map` rely entirely on `Table`               | None yet | Will involve new APIs       |
| Absorb `RawTable` into `HashTable` implementation | None yet |                             |
| Unsafe code auditing/additional refactoring       | None yet |                             |
| Move `hashbrown` into `alloc`                     | None yet | Will eventually require FCP |

## Team asks

| Team   | Support level | Notes                              |
| ------ | ------------- | ---------------------------------- |
| [libs] | Large         | Review API proposals, code changes |

## Funding

| Purpose        | Cost    | Funded | Sponsor(s) |
| -------------- | ------- | ------ | ---------- |
| Implementation | Unknown | No     |            |
