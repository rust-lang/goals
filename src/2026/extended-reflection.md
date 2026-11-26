# Extended reflection

| Metadata             |                                       |
|:---------------------|:--------------------------------------|
| Contact              | @yara-blue                            |
| Funding contact      | [Hexcat](https://hexcat.nl)           |
| Status               | Proposed                              |
| What and why         | Enable code analysis using reflection |
| Timespan             | 2026-2027                             |
| Roadmap              | Easily write code analysis            |
| Highlight            | Const and reflection                  |
| Zulip channel        | https://rust-lang.zulipchat.com/#narrow/channel/572285-goals.2Freflection |
| [compiler] champion  | @oli-obk                              |
| [lang] champion      | @nikomatsakis                         |
| [types] champion     | @oli-obk                              |

## Summary

Code analysis tools are currently separate tools outside the compiler. This has led to a few large code-analysis projects. We will extend the reflection API such that code-analysis can run as a compiler plugin during const-eval.

## Motivation

### The status quo

Code analysis is commonly done by building your own compiler-driver. That will then run as a separate tool in the build system. This has a few downsides:

* you need to run the compiler once normally, once for Clippy, once for your proof checker, once for every linter, ... etc.
* you need to use the internal API of rustc, which changes regularly (see also rustc_public).
* you need to update all your linters to new versions of the rustc libraries to use new language features.
* lints need to be aggregated in lint checkers instead of every crate just being able to ship its own.

In practice this has led to similar situations like with serialization, where one large project (in this case Clippy) is where all the lints live, instead of having a decentralized system where each crate can ship lints specifically for its users.

Instead of building a compiler-driver, some code-analysis tools ingest rustdoc-json. Though this limits what code-analysis can do given rustdoc-json does not contain type information. A well known tool taking this approach is cargo semver check.

There are four existing projects that try to solve this outside the compiler:
* [charon](https://github.com/AeneasVerif/charon) (ongoing)
* [dylint](https://github.com/trailofbits/dylint) (ongoing)
* [redpen](https://github.com/estebank/redpen) (abandoned 2 years ago)
* [marker](https://github.com/rust-marker/marker) (abandoned 3 years ago)

The projects charon and dylint are both compiler-drivers which provide an interface for code-analysis to plug into. This can only address some of the issues if one of them was universally adopted.

### What we propose to do about it

We already aim to stabilize a minimal reflection API (see [min-type-reflection](https://goals.rust-lang.org/2026/min-type-reflection.html)). We should extend that interface to expose enough of the compiler so code-analysis can be done in const-eval. This way analysis will run directly in the compiler.

To get there we will add `#[rustc_linttime]` functions as a measure to clearly separate functions that break semver, but are useful for linting (e.g. accessing arbitrary private fields and layout information). We may remove this later see the faq for why.

We will then create a way for registering `rustc_linttime` functions as new lints and create APIs for emitting lint warnings and errors from `rustc_linttime` functions.

With that in place we will extend the reflection API as needed for code-analysis, where needed limited to _linttime_. As a part of this we will:
- Design a maximally de-sugared minimal language representation for Rust code (of function bodies). We will demonstrate a purely textural representation against a structured representation (possibly by basing the textural representation on the structured representation).
- Expose APIs for performing various trait solver operations (such as projection normalization & proving bounds).

In the future as part of the [reflect-from-wasm](https://goals.rust-lang.org/2026/reflect-from-wasm.html) goal we want to expose the entire API to executable wasm blobs.

### Design axioms

- Small composable targeted methods, continuing what we have for [min-type-reflection](https://goals.rust-lang.org/2026/min-type-reflection.html). For example instead of adding `type_id.is_f32()` users use the existing `type_id.size()` and use `matches!(type_id.kind(), TyKind::Float)`. Then when Rust gets a `f16` type we need no new methods.
- Guide what demonstrators we develop by concrete use cases (e.g. existing lints or proof checkers).
- Build upon the work done by other projects such as _rustc-public_ and _charon_. 
- Avoid mixing semver-breaking APIs with the rest of the reflection interface.
- Create APIs that are clearly marked as prototypes (e.g. macros/attrs with placeholder names) for lint registration and emission, to avoid limiting future design work.

### The "shiny future" we are working towards

We're shooting for a future where it is easy to apply your own and third party code analysis like model checkers, lints and diagnostics to any Rust project. See the [roadmap-code-analysis](https://goals.rust-lang.org/2026/roadmap-code-analysis.html) for details.

This is currently an open ended goal, however with most of this goal in place users will be able to write code-analysis tools in const-eval. We will have a (limited) model-checker demonstrator which proof whether a piece of code can panic.

### Work items over the next year

| Task                                                         | Owner(s)   | Notes |
|--------------------------------------------------------------|------------|-------|
| Design a maximally desugared minimal language representation | @yara-blue |       |
| Create an API to register lints and emit warnings or errors  | @yara-blue |       |
| Build demonstrator and create needed reflection API          | @yara-blue |       |
| Author call for testing blog post                            | @yara-blue |       |
| Expand API further as needed by users                        | @yara-blue |       |
  

## Team asks

| Team       | Support level | Notes                                                    |
|------------|---------------|--------------------------------------------------------- |
| [compiler] | Medium        | Standard reviews                                         |
| [lang]     | Large         | Minimal language design                                  |
| [libs]     | Small         | Linter API review (prototype)                            |
| [types]    | Medium        | Trait solver API                                         |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | Ask | No | - |

## FAQ

### How does this not cause new semver hazards?

If code analysis panics or otherwise crashes, this will not cause compilation to fail. We will add a new deny-by-default lint that will be emitted instead of failing compilation. This means that any private changes inside a crate cannot cause downstream builds to fail to compile. The new lint will be emitted instead however it will be easy to disable.

### The effect of lints on upstream crates

Such extensive code-analysis available in the language enables adding features
that look like language features outside the compiler. These will inevitably
create some interesting scenarios. For example if an upstream crate has code-analysis enabled which emits a warning located in a crate further upstream.

#### example

Imagine authoring the crate `resampler` it depends on the `fft` crate which
in turn relies on `complex`. 

The `fft` crate uses code-analysis to ensure its API can never panic:

```rust
// in fft's lib.rs
use complex::Complex;

#[forbid_panicking]
pub fn fft(data: &[f32]) -> &[Complex<f32>] {
    // ...
    let c = Complex::from(a, b);
   //  ...
}
```

Now `Complex` put out a patch release which changes the `Complex::from` implementation it can now panic. I'd argue this is not a semver breaking, at least not in the current definition. Now you run cargo-update, the version of `Complex` in your tree is updated but now `fft` starts emitting lints. The lint emitted is easily ignored but should you? Did `fft` break? Did `complex`? Where do you rais an issue if you do at all.

We'll take this into account while designing and experimenting with the interface.

### Why even do any const-eval and not just Wasm blobs?

Easier experimentation at this stage without much cost to maintenance (considering `rustc_comptime` already exists it is trivial to add `rustc_linttime` extended).

We also want to compare running in const-eval to compiling to Wasm first, but that is part of the [reflect-from-wasm](https://goals.rust-lang.org/2026/reflect-from-wasm.html) goal.
