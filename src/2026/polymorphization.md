# Polymorphic code generation experiment


| Metadata            |                                                                    |
| :-----------------  | ------------------------------------------------------------------ |
| Contact    | @camelid                                                           |
| Status              | Accepted                                                           |
| [compiler] champion | @camelid                                                           |
| Tracking issue      | [rust-lang/goals#758]                                              |

## Summary

Implement experimental support in the Rust compiler for polymorphic code generation. Instead of monomorphizing generic code, the compiler will generate type-agnostic instances and add hidden vtable parameters where needed, which should reduce compile times and generated binary sizes.

## Motivation

### The status quo

One of the major issues facing Rust users is slow build times, especially during the development cycle. Check builds, which skip codegen, allow users to iterate faster to fix compile errors, but a full build is still needed to run tests or manually try out a change. In full builds, monomorphization is a significant contributor to slow compilation times. Each instance of a generic function must have trait solving and const evaluation performed on its monomorphized body and, more importantly, go through LLVM's entire codegen pipeline. As the performance of Rust's frontend improves through parallel-rustc and other initiatives, monomorphization and codegen will become an even bigger bottleneck since they are already heavily parallelized.

Another pain point is large binary sizes, not just in final release-mode binaries but also in (debug) artifact directories. The build directory for rust-lang/rust can easily surpass 100 GB, and I have seen extreme real-world cases of build directories containing up to 1 TB of data (often due to multiple configurations or rebuilds). Monomorphization is a significant contributor to these large binaries and build directories because it generates many instances per generic functions. It is possible to make use of link-time tools to eliminate instances that end up with identical code, but this requires extra analysis (further contributing to slow compile time) and only makes sense when generating final binaries for release.

In the past, there was another project, also called "polymorphization", led by @davidtwco that attempted to make progress on this issue. However, it was restricted to eliminating only "unused" parameters that didn't participate in codegen. Moreover, the project faced roadblocks to success relating to the detection of which parameters are actually unused. [`TypeId` and its breaking of parametricity][typeid-poly] was a contributing factor, and [closures added complications][closures-poly-orig] as well.

[typeid-poly-orig]: https://github.com/rust-lang/rust/issues/75325
[closures-poly-orig]: https://github.com/rust-lang/rust/issues/74636

### What we propose to do about it

I plan to implement support for truly polymorphic code generation in Rustc. Unlike the previous polymorphization project, my design works for unused and used generic parameters alike, significantly expanding its potential benefits and sidestepping issues with determining whether parameters are relevant to codegen. The axiom is "monomorphic data, polymorphic code". In other words, unlike languages like Haskell and OCaml that generally rely on uniform representations (boxing) to enable polymorphic code generation, data layouts will stay the same under polymorphization, and only function ABIs will change. Polymorphized instances can coexist with monomorphized instances, enabling incremental improvements to the kinds of functions supported by polymorphization. In fact, polymorphization operates at the (type) parameter level, so some parameters can be polymorphized (becoming "erased types"), while others remain monomorphized, within a single function.

To avoid runtime layout computations, Rustc will still monomorphize parameters based on their data layout. Thus, for example, if we have `fn foo<T>(...)`, then `foo::<&u32>` and `foo::<&String>` become the same instance since in both cases their parameter is just a pointer, with a niche at zero. However, `foo::<usize>` is a separate instance because it lacks a niche. This matters because `foo`'s signature, body, or callees may construct types like `Option<T>` that attempt to make use of the niche. We may also consider disabling the niche optimization when polymorphization is enabled to make more instances shareable.

Data layouts are not the only thing generic code needs to know about its type parameters. Trait methods and associated types also come into play, as do drop glue and `TypeId` information. The key is that all of this information already must be known statically at some point in the call graph. Thus, it can be threaded through implicit function arguments inserted during codegen. Essentially, each type parameter will generate a hidden extended-vtable argument containing the type's `TypeId`, its drop glue function pointer, and each of its trait methods' function pointers. Type parameters whose bounded traits have associated types will generate additional vtable arguments, one per associated type.

#### Prior art

Rust is somewhat unusual in its use of monomorphization for operationalizing generics. Other functional languages with polymorphic type systems, like Haskell and OCaml, usually rely on uniform representations (aka boxed, aka type erased) for data, which means that every type has the same ABI (essentially just a pointer) and polymorphic code can be generated. Most Java types also have a uniform representation based on pointers and vtables. C++ is most similar to Rust in that it heavily uses monomorphization via templates to implement generics.

In general, systems languages that prioritize runtime performance and zero-cost abstractions prefer monomorphization, while higher-level languages that may already have heavier runtimes such as garbage collectors prefer type erasure. In addition to the effects on runtime, compile-time, and binary size, the choice of generics implementation notably also affects the expressiveness of the type system, with type erasure enabling more powerful features.

However, there are also hybrid implementation strategies for generics, which are used by languages like Swift and Go. Both of these languages generate polymorphic code by default, with monomorphization (also called specialization) being an optimization that they may apply. [Swift threads hidden witness tables](https://faultlore.com/blah/swift-abi/) (similar to Rust vtables) through the generated code with information about each type parameter's ABI and methods available. [Go groups generated instances](https://github.com/golang/proposal/blob/58e952a6dd29795163bdff3de8b2d77329ff995b/design/generics-implementation-dictionaries-go1.18.md) by their type parameters' gcshape (how they appear to the allocator and the garbage collector). Go's gcshape is somewhat analogous to our concept of data layouts.

Recently, Swift has been walking back their embrace of full dynamism through the introduction of [Embedded Swift](https://www.swift.org/get-started/embedded/), which requires monomorphization and necessarily does away with language features like Library Evolution (ABI stability) that depend on dynamism. However, their focus is on supporting "escape hatches" from polymorphic code for the rare cases when developers need low-level control and maximum runtime performance. On a related note, there is a [research paper from the 1990s](https://dl.acm.org/doi/10.1145/143165.143205) that describes a prototype ML compiler that compiles code using unboxed representations as an optimization where possible.

Finally, there is currently an accepted Rust Project Goal for a [Dictionary Passing Style Experiment](https://goals.rust-lang.org/2026/dictionary-passing-style-experiment.html) in rustc, which is loosely related to this work. Currently, rustc recomputes the concrete impls to use at trait call sites in codegen. This project goal instead plans to compute this information ahead of time during type-checking and simply reuse it later during codegen. My work aims to eventually delay the _use_ of this information even further, to runtime, at least in debug builds.

### Expected outcomes

I estimate that the full version of polymorphization that I am working toward could speed up post-mono trait solving, codegen, and linking by between 30% and 2x, by eliminating up to half of generated instances in many cases. Binary sizes should also be reduced by a similar degree as a result, without requiring link-time optimization, helping shrink the size of artifact directories containing debug builds. As mentioned above, incremental progress can be made toward this full vision, providing checkpoints with smaller but noticeable speedups and binary size reductions.

### Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Implement basic polymorphization proof-of-concept | @camelid | Partially working already on real-world code like `std` |
| Support `TypeId`, drop glue, and similar features | @camelid | |
| Support simple dyn-compatible traits | @camelid | |
| Support some non-dyn-compatible traits | @camelid | |

### Target timeline

Assuming full funding starting in Month 1. Timeline is looser without funding, and either way may be affected by external factors like team availability.

* Month 1-3:
  * Split out and land PRs from the current draft to lay groundwork
  * Rewrite MIR calls during polymorphization so that "sidecars" (like supercharged vtables) can be added to function calls
  * Start passing sidecars for drop glue, `TypeId`, and anything else observable on stable Rust that breaks parametricity (i.e. is related to the underlying type but doesn't appear as a bound)
  * Figure out exclusion rules to avoid polymorphizing generic functions with non-Rust ABI
  * Open compiler MCP for adding `-Zpolymorphize`
  * Land working but highly-experimental `-Zpolymorphize` flag (may cause unsoundness and should be used with caution like any incomplete feature)
* Month 4-6
  * Determine minimal subset of trait bounds or other bounds that can be supported in sidecars
  * Implement support for those trait bounds
  * Determine how to correctly deal with specialization that affects stable code (e.g. originating in `std`)
  * Investigate polymorphizing const parameters
* Month 7-9
  * Extend to more traits and bounds
  * Evaluate compatibility with other unstable features
  * Address design or implementation issues that arose
* Month 10-12
  * Continue to polish
  * Move toward stabilization


## Team asks


| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | medium        | reviewers: @davidtwco, @bjorn3, @oli-obk, @workingjubilee, @cjgillot |
| [types]    | small         | vibe check for erased types and which trait bounds to support |


## Funding


| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor (part-time) | Ask | No | |
| Reviews | TBD | No | |


## Frequently asked questions

### How does this interact with specialization?

Specialization in and of itself does not conflict with polymorphization. However, specialized _blanket impls_ do cause problems. I am actively working with @nikomatsakis and @jackh726 to figure out the best path forward. The design of specialization itself is in flux, and the new designs being explored will likely work better with polymorphization. Unlike the current specialization design, they avoid breaking the type system parametricity that is assumed by polymorphization.
