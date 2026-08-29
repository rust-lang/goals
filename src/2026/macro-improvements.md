# Declarative (`macro_rules!`) macro improvements

| Metadata         |                                    |
| :--              | :--                                |
| Contact | @joshtriplett                      |
| Status           | Accepted                           |
| Tracking issue   | [rust-lang/goals#629] |
| Zulip channel    | #t-lang                            |
| [lang] champion  | @joshtriplett                      |


## Summary

I propose to continue improving the state of declarative macros (e.g.
`macro_rules!` macros) in Rust, with the eventual aim of making them just as
capable as proc macros while being easier to write and faster to compile.

I'll also prototype extensions to the declarative macro system to make macros
easier to write. I intend to collaborate with the ["Reflection and Comptime"
goal](https://goals.rust-lang.org/2026/reflection-and-comptime.html),
and work with experimental implementations of comptime when available.

The initial focus of this goal is the stabilization of declarative attribute
and derive macros, along with enough functionality to enable the adoption of
such macros by a reasonable subset of crates without substantive degradation of
functionality or usability.

## Motivation

This project goal will make it possible, and straightforward, to write any type
of macro using the declarative `macro_rules!` system. This will make many Rust
projects build substantially faster, make macros simpler to write and
understand, and reduce the dependency supply chain of most crates.

### The status quo

In stable Rust, attribute macros and derive macros can still only be written as
proc macros. To address this, I've previously implemented [declarative
attribute macros](https://github.com/rust-lang/rust/issues/143547) and
[declarative derive macros](https://github.com/rust-lang/rust/pull/145208).
These are available in nightly Rust, but not yet stable.

Given the popularity of workarounds such as the
[`macro_rules_attribute`](https://crates.io/crates/macro_rules_attribute)
crate, we know that some users will want to use these features as soon as
they're available in stable. Other users may still wish to use proc macros even
if these features exist in declarative macros, because some macros are simpler
to implement using the full power of Rust code, and may be painful to refactor
into purely declarative form.

On the other hand, proc macros are complex to build, have to be built as a
separate crate that needs to be kept in sync with your main crate, add a heavy
dependency chain (`syn`/`quote`/`proc-macro2`) to projects using them, add to
build time, and lack some features of declarative (`macro_rules!`) macros such
as `$crate`. Proc macros also make `cargo check` slower, because proc macros
require a full build of the proc macro crate and its dependencies.

As a result, proc macros contribute to the perceptions that Rust is complex,
has large dependency supply chains, and takes a long time to build. Crate
authors sometimes push back on (or feature-gate) capabilities that require proc
macros if their crate doesn't yet have a dependency on any, to avoid increasing
their dependencies.

### What we propose to do about it

I first intend to write clear documentation stating that projects may need
additional functionality before they want to switch to declarative macros, and
users should not push ecosystem crates excessively hard for rapid switches from
proc macros to declarative macros. This documentation will help improve the
effect of these features on the ecosystem, and avoid causing undue hardship for
the maintainers of proc macro crates by inviting pressure to port such macros
without sufficient replacement capabilities.

After writing that documentation, I intend to propose stabilization of
declarative attribute and derive macros, and address any issues that block
stabilization.

Concurrently, I will continue to work towards additional macro features,
including additional work on macro fragment fields (aligning with the planned
work on reflection), and comptime `macro fn` macros. By the end of the year, I
hope to have functional prototypes/experiments for some approaches in this
area, and possibly RFCs based on those experiments. I'll measure the success of
such experiments by how well they handle two common patterns: iterating over
the fields of a struct, and writing conditionals based on whether a pattern is
or isn't matched.

I also hope to seek fellow travelers interested in macros, and aggregate a team
of experts to collaborate on this work.

I expect macro improvements to be a long incremental road, with regular
improvements to capabilities and simplicity. Crate authors can adopt new
features as they arise, and transition from proc macros to declarative macros
once they observe sufficient parity to support such a switch.

Eventually, I aim to make it possible for the vast majority of crates to never
need proc macros. Crates should have the capabilities to easily implement
attributes, derives, and complex macros using exclusively the declarative
`macro_rules!` system. Furthermore, crate authors should not feel compelled to
use proc macros for simplicity, and should not have to contort their procedural
logic in order to express it as a declarative macro. Crate authors should
have the option to write macros using `macro_rules!` in either a recursive or
semi-procedural style. For instance, this could include constructs like `for`
and `match`.

I expect that all of these will be available to macros written in any edition,
though I also anticipate the possibility of syntax improvements unlocked by
future editions, or within future macro constructs such as `macro fn`.

## Design goals

- Incremental improvements are often preferable to a ground-up rewrite. The
  ecosystem can adopt incremental improvements incrementally, and give feedback
  that inspires further incremental improvements.
- There should never be a capability that *requires* using a proc macro.
- The most obvious and simplest way to write a macro should handle all cases a
  user might expect to be able to write. Where possible, macros should
  automatically support new syntax variations of existing constructs, without
  requiring an update.
- Macros should not have to recreate the Rust parser (or depend on crates that
  do so). Macros should be able to reuse the compiler's parser. Macros
  shouldn't have to parse an entire construct in order to extract one component
  of it.
- Transforming iteration or matching into recursion is generally possible, but
  can sometimes obfuscate logic. Users should be able to write macros using
  iteration, matching, recursion, or any combination thereof, whichever they
  feel is most appropriate for the problem.

## Work items over the next year

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Write user documentation for attribute and derive macros | @joshtriplett | |
| Write careful caveats for crate users to discourage demands for rapid adoption | @joshtriplett | |
| Write announcement blog post for attribute and derive macros | @joshtriplett | Will include references to the two previous items |
| Stabilize attribute and derive macros | @joshtriplett | I do not anticipate this requiring additional design meetings. |
| Further experiments on macro metavariable expressions and macro fragment fields | @joshtriplett | |
| Preliminary design for new `macro fn` mechanism | @joshtriplett | |
| Design for how `macro fn` and `macro_rules!` can interoperate syntactically | @joshtriplett | |
| Collaborate with ongoing work on `comptime` and reflection | @joshtriplett | |

## Team asks

| Team       | Support level | Notes                                                                            |
| ---------- | ------------- | -------------------------------------------------------------------------------- |
| [lang]     | Medium        | This is a stabilization, but we have previously explored the design in detail, and it's simple and straightforward. It should be able to take place asynchronously. Nonetheless, I can upgrade this to "Large" if people believe it rises to that level. |

## Frequently asked questions

### What about "macros 2.0"

Whenever anyone proposes a non-trivial extension to macros, the question always
arises of how it interacts with "macros 2.0", or whether it should wait for
"macros 2.0".

"Macros 2.0" has come to refer to a few different things, ambiguously:

- Potential future extensions to declarative macros to improve
  hygiene/namespace handling.
- An experimental macro system using the keyword `macro` that partially
  implements hygiene improvements and experimental alternate syntax, which
  doesn't have a champion or a path to stabilization, and hasn't seen active
  development in a long time.
- A catch-all for hypothetical future macro improvements, with unbounded
  potential for scope creep.

As a result, the possibility of "macros 2.0" has contributed substantially to
"stop energy" around improvements to macros.

This project goal takes the position that "macros 2.0" is sufficiently nebulous
and unfinished that it should not block making improvements to the macro
system. Improvements to macro hygiene should occur incrementally, and should
not block other improvements.

### Could we support proc macros without a separate crate, instead?

According to reports from compiler experts, this would be theoretically
possible but incredibly difficult, and is unlikely to happen any time soon. We
shouldn't block on it.

In addition, this would not solve the problem of requiring proc macros to
recreate the Rust parser (or depend on such a reimplementation).

### Does `comptime` or reflection subsume this?

No, `comptime` and reflection *complement* this.

A `comptime` system would get part of the way to running Rust code at macro
evaluation time, but would require additional work to run at that point in the
compiler.

Reflection APIs would allow walking types and fields and similar at runtime,
and we could mirror those same APIs at compile time, but each of those two use
cases requires substantial work and design.
