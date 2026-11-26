# Easily write code analysis

| Metadata | |
|:--|--|
| Short title |  Easily write code analysis |
| What and why | There are invariants which cannot be expressed in the type system, support user written code-analysis in the compiler to cover some of that |
| Contact | @yara-blue |

## Summary

Allow users to run, build and distribute code analysis tools with the same ease as crates. Making it practical to apply your own and third party model checkers, lints and diagnostics to any Rust project.

## Motivation

### The status quo

Even with great design crates cannot express all their invariants in the type system instead relying on documenting. Inevitably users will misread or misunderstand that documentation. Some of these invariants could be verified by tooling, and there is great tooling out there, most notably clippy. Each tool however runs outside the compiler and needs its own setup. This has lead to only a few tools being developed and limited adoption outside clippy. These are not the place for project specific code analysis.

A newer development is users instructing an LLM to uphold a list of lints/rules, though this is non-deterministic and generally not as efficient as dedicated tooling.

There are niche features highly valuable to a small group of users that we cannot add to Rust as the rise in complexity is not worth it (see: [minus 100 points](https://learn.microsoft.com/en-us/archive/blogs/ericgu/minus-100-points)). There are also complex features we want in the compiler but take long to stabilize. With sufficient access to the compiler some of these could be implemented outside the language and be available to everyone immediately, though opt-in. Examples would be an equivalent to  c++'s _noexcept_ or the concept of _pure functions_.

### What we are shooting for

We're aiming for a future where crates use and create custom code-analysis with the same ease as they currently use dependencies. They could for example use code-analysis to guard invariants, improve diagnostics and/or uphold project specific coding guidelines. Modular third party code-analysis is readily available and there are supporting crates providing basic building blocks that make it easy to write your own. Existing tools like [Charon](https://github.com/AeneasVerif/charon) and model checker [Kani](https://github.com/model-checking/kani) are available as such building blocks. Ready made code-analysis tools are available on crates.io and ship pre-compiled Wasm. They may use a similar interface to lints but can be as complex as required, for example they may resort to calling external proof checkers. The code-analysis can run as part of cargo check and plays well with incremental compilation. Generally code-analysis won't impact total compilation time as we can schedule it parallel to LLVM-codegen at the cost of increased peak memory usage and CPU utilization.

As code-analysis cannot modify source code and therefore cannot negatively affect the correctness of the project they are applied to (outside a false sense of security in the case of false negatives). This makes project specific analysis tooling a candidate for accelerated development independent of the rest of a projects codebase.

With such powerful code-analysis in crates there will be features that can be experimented with in crates instead of requiring changes to the compiler. No RFCs or MCPs needed, there can be multiple implementations and there would be no need to use a nightly compiler to use these.

### Design axioms

* *Easy to use* In contrast to external tools we can provide a non-intrusive interface and make it trivial to add code-analysis leading to widespread use of such tools.

* *Extend existing endeavors* Where possible we make our work useful for or build on other projects (e.g. proc macro Wasm support, rustc_public, ...).

* *Experiment, experiment, experiment* We're confident in the approach, however there are too many unknowns to start writing RFCs at the start. Changes (even significant ones) are expected to occur frequently.

* *Guided by users* Identify the shortest path within our dream of the ideal reflection system that adds real value to users, then build that. Rinse, repeat.

* *Minimal interface* Where it makes sense implement functionality in supporting crates. This empowers anyone to contribute towards finding a great high level API. We'll revisit this before stabilization and decide which high level APIs should be moved into the compiler.

### How we get there

| Goal | Timespan | What and why |
| --- | --- | --- |
| (((ROADMAP ROWS: Future of code analysis))) |
| Cargo & Crates.io integration | Future | Easily distribute code analysis tools |

`Reflect from Wasm` can be worked on in parallel to `Extended reflection` and `Min type reflection`, as it only enables the use of the APIs developed in those goals, without relying on which APIs exist there.

## Funding

(((FUNDING TABLE: Future of code analysis)))

## Frequently asked questions

### Existing endeavors 

What is the existing work that we will speed up or build on?

- The minimal reflection API is being developed as part of the `constify-all-things` roadmap. 
- We've long wanted Wasm in the compiler to speed up proc macros through pre-compilation. There are existing experiments around that which we can build on and this work can then be extended by them. This applies both to having a Wasm runtime in the compiler and the cargo/crates.io infrastructure to conveniently distribute Wasm blobs.
- rustc_public solves similar problems, but notably requires separate tooling. We will built on their work, as it is already decoupled from rustc internals. We can use this decoupling to build a `Reflect from Wasm` backend for rustc_public.

### What could this look like?

We will experiment with the interface to figure out what works, the below is just a sketch the real thing most likely won't look like this:

```rust
#[linting_crate::forbid_panicking]
fn should_deploy_airbags() -> bool {
    // complex safety critical code
}
```

And maybe we'd write this tool like this:
```rust
#[item_lint]
fn forbid_panicking(item: core::any::ItemId) -> Result<(), LintError> {
    let ItemKind::Function(item) = item else {
        return Err("Can only be used with functions");
    };

    // proof checking
}
```

On the other hand maybe we need these to be in a special crate type? Maybe the attribute should specify what it can be called on? Should `use` statements apply to these? Is it enough to annotate main or do we need crate or mod level interfaces? These are just some of the questions we'll encounter as we experiment.
