# Reflect-from-Wasm

| Metadata             |                                    |
|:---------------------|:-----------------------------------|
| Contact              | @yara-blue                         |
| Funding contact      | [Hexcat](https://hexcat.nl)        |
| Status               | Proposed                           |
| What and why         | Code-analysis in Wasm through reflection |
| Timespan             | 2026-2027                          |
| Roadmap              | Easily write code analysis         |
| Highlight            | Const and reflection               |
| Zulip channel        | https://rust-lang.zulipchat.com/#narrow/channel/572285-goals.2Freflection |
| [compiler] champion  | @oli-obk                           |
| [lang] champion      | @nikomatsakis                      |
| [libs] champion      | ?????????                          |


## Summary

Using the reflection API users can write code-analysis in compile time only functions. These functions are limited in both performance and ergonomics. We will build a convenient interface for applying code-analysis and make it possible to run that code-analysis fast through Wasm running in the compiler.


## Motivation

### The status quo 

The reflection API (see [min-type-reflection](https://goals.rust-lang.org/2026/min-type-reflection.html) and [extended-reflection](https://goals.rust-lang.org/2026/extended-reflection.html)) enables users to write their own code-analysis tooling like linters and model checkers and run these inside the compiler. This means we no longer have to build and run separate tools, and when authoring code-analysis you can easily reuse builing blocks from other tools through crates-io. Reflection is available on nightly now and can already be used for simple lints, for example limiting the length of an array.

Code-analysis being restricting to const-eval handicaps it, given:
- Const-eval is inherently limited in what it can evaluate. There are no statics, and lots of language-features are still unstable (no traits) and will be so for a long time. This means large parts of the ecosystem do not support const.
- Const-eval is not designed for speed, it focuses on type system correctness. While execution speed will not be noticeable for simple applications we want users to be able to write performant complex code-analysis such as model-checking. A similar performance consideration exists for some use cases of const eval. For example converting assets like images to a common format during compilation these can then be made part of the binary.

While this may be resolved by adding the ability for const-eval to call out to external tools that has other downsides:
- External tools can have complex runtime dependencies (e.g. dylibs written in other languages) and can be platform dependent. This makes them hard to use.
- An external tool may access the system in all kinds of ways that we cannot easily track. These side effects make it hard to cache their invocation. This makes their usage slow down compilation too much.

We've long wanted Wasm in the compiler to distribute proc macros and build scripts as pre-compiled Wasm blobs to speed up builds. There is a [pre-rfc](https://internals.rust-lang.org/t/pre-rfc-sandboxed-deterministic-reproducible-efficient-Wasm-compilation-of-proc-macros/19359) and was a [discussion](https://github.com/rust-lang/all-hands-2026/issues/73) about this at the all-hands this year. The latest work on this front are two draft PRs each integrating a different Wasm runtime, both efforts have been stalled. 
 
### What we propose to do about it

Experiment with adding Wasm based compiler plugins. These would be triggered from a new code-analysis interface and from const-eval directly. The new interface would allow the Wasm code access to the reflection API for code-analysis. Since Wasm is optimized in LLVM and Wasm-opt and Wasm interpreters often support JIT to speed up frequently executed Wasm code, this can result in near native performance. This enables writing fast code-analysis in normal Rust without the limitations of const-eval. In addition tooling written in other languages can be compiled to Wasm as well. The reflection interface would then function like a small shim between that tooling and the compiler.

We will reuse and build upon the exploration done for proc macros. We will make sure to add a Wasm runtime such that it can easily be extended for later use to speed up proc macros and build-scripts. 

### Design axioms

- Postpone binding ourselves to a Wasm runtime by imposing a clear architectural boundary. There are multiple runtimes actively being developed and what seems like a good option now may be abandoned in the future. Let's not delay or spend time on evaluating these when a little extra work can make it easy to change later.
- Keep the Wasm infrastructure general so it can be used by proc macros later. Adding pre-compiled Wasm for proc macros is explicitly not part of this goal.
- Limit ourself to rustc. While Cargo and Crates.io integration are essential to reach the shiniest future we are working towards, they are explicitly out of scope for this experiment.
- Take small steps, this is very new - we expect to change direction a few times. For example a lint that always shows an error is a good sub-goal for setting up the runtime and linting interface.
    - Use demonstrators from earlier reflection work. Specifically `expanded-reflection` aims for a demonstrator which checks some code for panics. Running that from Wasm is a good milestone.
    - Do not aim to compile an existing proof checker. That would couple this goal tightly to extended-reflection. It however would greatly benefit from this goal being completed when prioritizing which APIs to land first.

### The "shiny future" we are working towards

We're shooting for a future where it is easy to apply your own and third party code analysis like model checkers, lints and diagnostics to any Rust project. See the [roadmap-code-analysis](https://goals.rust-lang.org/2026/roadmap-code-analysis.html) for details.

When this goal has been completed:
- Anyone can write a code-analyzer and distribute to any platform.
- Code-analysis can be applied at any level: from analyzing a whole crate to just a specific function.
- Const-eval functions and code-analysis can be shared as Wasm blobs that are faster to run and don't incur extra build time
  since they do not need to be compiled to MIR.

We can then build an even shinier future where:
- Using third party code-analysis is as simple as adding a dependency in Cargo.
- The compiler speeds up repeated code-analysis in the same way as it does for builds: by tracking and caching Wasm file access and network-io at the user's discretion. We may even start a Wasm blob at the point where it's inputs changed instead of fully rerunning it.


## Work items over the next year

| Task                                                     | Owner(s)   | Notes |
|----------------------------------------------------------|------------|-------|
| Design the code-analysis interface                       | @yara-blue |       |
| Write a rustc-Wasm-runtime crate                         | @yara-blue |       |
| Hook up the runtime to the new interface                 | @yara-blue |       |
| Write Wasm <-> reflection interface                      | @yara-blue |       |
  

## Team asks

| Team       | Support level | Notes                                                    |
|------------|---------------|--------------------------------------------------------- |
| [compiler] | Large         | Standard reviews                                         |
| [lang]     | Medium        | Design meeting, experiment                               |
| [libs]     | Medium        | Reviews, design opinions                                 |


## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | Ask | No | - |

## FAQ

### Caching Wasm blob side effects

We can track which query results a Wasm blob accessed since we control the reflection API. Given we also control the runtime we may as well inspect any system access like file-IO. With this information we can figure out if any of the inputs to a blob changed and it should be re-run or that we can reuse a cached result instead.

We may even take this one step further and provide API's for blobs to cache intermediate results so they can do incremental work when possible. 

### Wasm <-> reflection interface

All reflection methods are thin abstractions around intrinsics where the actual interface to the compiler lives. These intrinsics are dispatched by the const-eval machine and not available at runtime.

A code-analyzer running in the compiler through Wasm needs access to these intrinsics evaluated in the context of the crate it is analyzing. We can use a Wasm concept called _host functions_ for this. These are defined outside WebAssembly and allow Wasm code to interact with its environment, we will use these as a bridge to the reflection intrinsics. This is similar to the existing proc macro bridge except this bridge passes queries to the compiler.

Figuring out how exactly we will do this and present it to users is a key part of this goal. It would be ideal if users could call the same reflection methods as in const-eval. These would then use the Wasm bridge to run the const-eval intrinsics for the crate being compiled. We will need to restrict calling the reflection methods outside of const to code-analyzers as they would be undefined symbols when not ran in our Wasm runtime.
