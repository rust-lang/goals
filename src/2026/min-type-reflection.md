# min type reflection

| Metadata             |                                    |
|:---------------------|:-----------------------------------|
| Contact              | @yara-blue                         |
| Funding contact      | [Hexcat](https://hexcat.nl)        |
| Status               | Accepted                           |
| What and why         | Compile-time type reflection to make `serialize(&my_struct)` work without derives |
| Timespan             | 2026-2027                          |
| Roadmap              | Constify all the things            |
| Tracking issue       | [rust-lang/goals#406]              |
| Other Tracking issue | [rust-lang/rust#142577]            |
| Highlight            | Const and reflection               |
| Zulip channel        | [#goals/reflection](https://rust-lang.zulipchat.com/#narrow/channel/572285-goals.2Freflection) |
| [compiler] champion  | @oli-obk                           |
| [lang] champion      | @nikomatsakis                          |
| [libs] champion      | @joshtriplett                      |


## Summary
Enable anyone to write general purpose functionality that works on any data type from any crate. We do this by adding compile time only functions to the language. With these we add functions which query the compiler directly during constant evaluation. We aim to stabilize the minimum set of functions required for introspection applications like serialization and logging. Expanding reflection beyond this will be a separate goal.

## Motivation

Creating new general purpose crates (like serialization crates, log/tracing crates, game engine state inspection crates) that should work with almost all other data structures is nontrivial today.
You either need to locally implement your new traits for other (common) crates, or the other crates need to depend on you and implement your traits.
This often hinders rollout and will never reach every crate. Most crate maintainers do not want to depend on 2+ serialization crates, and 3+ logging crates, so they will instead pick one,
causing everyone to either pick the large popular crates or be limited in what they can serializer/log. This is a hindrance to innovation and will (imo) long term cause the ecosystem to
stop evolving even when an objectively better solution to a problem is found.

A lot of crates also expose features for deriving various common ecosystem traits to save on compile time when those derives are not needed. With reflection, only the actual usage costs compile time, and only for those types that actually get processed. Similarly embedded devs want to suppress all the `Debug` derives as they blow up binary size significantly.

Reflection offers a way out of this dilemma, as you can write your logic for all types.
You would be processing the type information at runtime (or even preprocess it at compile-time, e.g. in const blocks) without requiring trait bounds on your functions or trait impls anywhere.
This means no one but consumers of your serialization/logging/game-engine will need to know about your crate, and you do not need the entire crates.io ecosystem to add derives for your traits.
Your consumers immediately are able to interoperate with tuples of any size, arbitrary structs and enums from arbitrary crates that neither depend on yours nor you on theirs.

#### Design axioms

* Prefer procedural const-eval code over associated const based designs (see also "why not uwuflection" in the FAQ).
    * We picked `const fn` in general evaluation over associated const based designs that are equally expressive but are essentially a DSL
* Favor small targeted queries over an ergonomic interface. 
* Ensure privacy is upheld, modulo things like `size_of` exposing whether new private fields have been added
    * This is important to ensure that we cannot break abstractions. Anything breaking that is out of scope for min-reflection, but we're separately considering how such things could be enabled for different use cases that do not cause ecosystem problems.
    * e.g. do not expose method ordering, documentation text, ...

#### The "shiny future" we are working towards

We expect there to be simple building blocks on crates.io for various applications, that can be mixed and matched to build exactly the kind of serialization/introspection scheme desired. This also pushes most of the compile-time effort to the crates actually performing such serialization/introspection, while still being organizable in a separate crate to improve caching.

Everyone can write a library for serialization, debug printing, runtime-introspection, ... that works on all public fields of Adts, and definitely works on all other types.

Runtime reflection crates like `bevy_reflect` and `facet` will still exist, but they essentially just differ in their user experience.

Furthermore it opens up new possibilities of reflection-like behaviour by
* specializing serialization on specific formats (e.g. serde won't support changing serialization depending on the serializer  [serde-rs/serde#2877]),
* specializing trait impl method bodies to have more performant code paths for specific types, groups of types or shapes (e.g. based on the layout) of types. 

Once we have reflection providing a stable interface to the compiler, we can extend that interface to answer deeper questions about what is being compiled. This could power linters and proof checkers running as compiler plugins. Both extending the reflection interface and enabling it's used by compiler plugins are separate project goals.

### Status quo

* `#[rustc_comptime]` attribute making a normal function only callable from const blocks or const/static item initializers, or other `#[rustc_comptime]` functions
* Various `#[rustc_comptime]` methods on `TypeId` that expose information about the type represented by that `TypeId`
    * Nested information (about fields or variants) has its own `FieldId` or `VariantId` types
    * Functions to instantiate a shim for a specific type and return a function pointer to the "generated" function (this is used to access discriminants at runtime).
* There are demonstrators showing that you can use these reflection APIs to debug or serialize any type without any trait bounds.

### What we propose to do about it

* stabilize minimally useful subset of reflection next year
* stabilize comptime functions and inherent methods next year

### Work items over the next year

| Task                                                     | Owner(s)   | Notes |
|----------------------------------------------------------|------------|-------|
| Flesh out the minimal API more                           | @yara-blue |       |
| Workout privacy and semver considerations                | @yara-blue |       |
| Author call for testing blog post                        | @yara-blue |       |
| Demonstrate uses in existing and new crates              | @yara-blue |       |
| Get T-lang feedback on current design                    | @yara-blue |       |
| Get T-libs feedback on API surface                       | @yara-blue |       |
| Stabilize                                                | @yara-blue |       |


## Team asks

| Team       | Support level | Notes                                                    |
|------------|---------------|--------------------------------------------------------- |
| [compiler] | Medium        | Standard reviews                                         |
| [lang]     | Large         | Design meeting, Stabilization                            |
| [libs]     | Medium        | ACP discussion & approval                                |
| [types]    | Small         | General discussion on any additional type-system changes |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | Ask | No | |

## Frequently asked questions

### Why do you need comptime in addition to reflection?

If we had a `bevy_reflect::Type` type in libcore and a

```rust
const fn type_of(id: TypeId) -> &'static Type;
```

function, that has the special requirement of, unlike every other `const fn`, not being callable at runtime, then we could work with type descriptions from normal procedural rust code.

So for this impl we would do

```rust
#[rustc_comptime]
const fn type_of(id: TypeId) -> &'static Type;
```

These functions can't be run at runtime, because that would require there to be some global table somewhere that maps all `TypeId`s to their repr. This is an obvious no-go.

### Why not continue where uwuflection left off?

See https://soasis.org/posts/a-mirror-for-rust-a-plan-for-generic-compile-time-introspection-in-rust/ for details on what uwuflection is

#### Structural processing

it makes procedural processing of type information very hard. E.g. to get the 3rd element of a tuple you need to 

```rust
<introwospect_type::<YourType> as FieldDescriptor<3>>
```

so to compute that index you need a constant. you can't just use a for loop to iterate over the indices.

we chose const fn over associated consts and generics which could compute the same thing as the const fn, just more expensively and mostly purely functional. so going to assoc consts again seems like it goes against that

This can somewhat be resolved by adding a compile-time for loop feature to the language, that will just expand the body N times in a macro/loop-unrolling kind of way.

#### generic const exprs

in order to use uwuflection in types in generic code you need to either write infallible code with min const generics or you need to add uwuflection bounds (Lots, think typenum) , which defeats the purpose.

### Why not go full zig-style comptime?

zig's approach to comptime from a very high level is effectively

* generate AST for all source files
* pick the `main` function and start compiling it and looking for what it needs to be compiled
* if a comptime function call is found, look only for what code that needs to compile, compile it and produce the resulting code of the comptime function
* continue the main compilation, which may now invoke the generated code and start compiling that

we do not experiment with this approach at this time, because the compiler is not set up in a way to permit proc macros from accessing type information from the current crate.
While there are ongoing refactorings that go into the direction of potentially allowing more of that in the future, that future seems to be more than 5 years away at my best guess.

* the compiler is not set up to add AST nodes while type information is already available. It possibly never will, and it would be an immense amount of work to get there. I'm doing lots of refactorings that would need to be done for something like that anyway, even if the goal is just better incremental and general compiler architecture.
* there are too many open language questions about it that we haven't even started to discuss
* a working comptime reflection implementation that works for most types and that works with regular const eval exists right now, so pursuing the definitely possible implementation will pay off in a shorter term.

### Tradeoffs with derives

Reflection is orthogonal to derives as they solve similar problems from different directions. Reflection lets you write the logic that processes your types in a way very similar to dynamic languages, by inspecting values' types during the execution of the reflection code, while derives generate the code that processes types ahead of time. Proc macros derives have historically been shown to be fairly hard to debug and bootstrap from scratch (we should totally also improve proc macro workflows). While reflection can get similarly complex fast, it allows for a more dynamic approach where you can easily debug the state your are in, as you do not have to pair the derive logic with the consumer logic (e.g. a serializer) and are instead directly writing just the consumer logic.

Derives can generate the code ahead of time, but don't have type information to do so, while reflection doesn't generate code, but instead allows you to build an interpreter for the minimal subset of rust metadata that you need, which often optimizes down to the ideal target machine code. It is also possible to special case the logic for individual types or types implementing specific traits, in case manual hot path optimizations are desirable.

### Future ergonomic API

The entry point to reflection is methods on `TypeId`. For various reasons, we're making the API fine grained and targeted. We'll end up with a wide and shallow API with many methods on `TypeId`. This is not nice to use directly, however we intend for end-users to use an ergonomic wrapper crate. We'll ensure there is at least one of those. We're doing this for the usual reasons (1).

At some point, potentially many years in the future (2), the ecosystem may coalesce on a single nice ergonomic wrapper for all use cases. If we end up stabilizing many reflection methods on `TypeId` any future API using `TypeId` as the entry point will have it's methods drowned out by the existing stable ones. Instead we can keep `TypeId` "clean" at the cost of worse ergonomics when not using a wrapper crate now.

(1): faster iteration, more accessible to people outside the compiler, less code in the compiler, avoid stabilizing a nice API that turns out to be bad later etc etc.
(2): we may even have reflection synthesize new code at some point. That could very well change what an ergonomic API looks like.
