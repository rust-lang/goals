# End-to-End Executable Rust Specification

| Metadata             |                                                                           |
| :--                  | :--                                                                       |
| Contact     | @Nadrieril                                                                |
| What and why         | Build an executable specification for Rust that covers the whole language |
| Status               | Accepted                                                                  |
| Tracking issue       | [rust-lang/goals#749]                                        |
| Zulip channel        | N/A                                                                       |
| [formality] champion | @nikomatsakis                                                             |

## Summary

The goal is to build a specification for the whole of Rust,
from surface syntax down to detailed semantics,
that is both legible and executable.

As design axioms, we want this specification to be:
- Understandable by non-experts, serving as a reference that users can reach for;
- Executable, so we can test it and ensure correctness;
- Easy to evolve, so it can be at the heart of language evolution;
- Well-structured, so it can be turned into maths and be amenable to formal analyses.

We propose that this specification would be a program written in a Rust-flavored literate language
(probably [specr]), interspersed with English explanations.
As a program, it would be an interpreter capable of executing real Rust source files.
As a specification, it would describe the meaning of the programs supported.

The plan is to build on top of the existing executable specification efforts,
namely [a-mir-formality] and [MiniRust],
by bridging the gap between surface Rust syntax
and these semantic models.
We propose to bridge that gap by successively desugaring
expressive syntactic constructs down to more basic ones,
as sketched in [rust-via-desugarings].

This will be a multi-year effort.
For this goal period the focus will be
on putting the bases in place,
namely parsing, integration with a-mir-formality/MiniRust,
tests, and fully fleshing out a couple features.

## Motivation

### The status quo

The official source of truth for "what the Rust language is" is a document written in English:
the Rust Reference (and to some extent the standard library docs).
Whilst these are well-crafted and precise documents,
English text is limited in how precise it can be,
and cannot be directly tested on actual programs.

As the language settles and its usage grows, the need for
precision and accuracy in its definition grows too.
Among others language designers, researchers, safety critical companies and unsafe code authors
would all benefit from a definition of Rust that is authoritative, unambiguous, and accurate to
what's implemented in the compiler.

There are currently two initiatives with the explicit goal
of making some parts of Rust semantics executably-precise:
- [MiniRust], which focuses on runtime aka dynamic semantics, for the purpose of defining what
  is/isn't UB;
- [a-mir-formality], which focuses on type-system aka static semantics, such as trait solving and
  borrow-checking. It specifies a higher-level language than MiniRust, and uses MiniRust for the
  dynamic semantics aspects.

Both have shown success, and MiniRust in particular is getting sufficiently complete and settled to be
a candidate for becoming official source-of-truth.

It's unclear how we'd do that however:
we would first need to precisely define how to get from surface Rust syntax down to these semantic models.
That's where this project comes in.

### What we propose to do about it

The approach we're putting forward is one based on successive desugarings.
I have previously written an [mdbook][rust-via-desugarings] that sketches
the successive desugarings we'd need to go from surface Rust down to something like a-mir-formality.

Here is how we propose the full spec would look like:
- Concretely, the spec would be a repo containing an mdbook written in literate Rust;
- The first chapter would define the AST for the language, along with the grammar to parse it,
  reusing the grammar definitions in the Reference;
- The second chapter would implement successive desugarings, following the sketch laid out in
  [rust-via-desugarings];
- Each desugaring step would come with English explanations and selected before/after examples[^1];
- Likely we may find it useful to define some intermediate languages that resemble the compiler's
  HIR/THIR at appropriate stages in the desugarings.
- The final section of the desugarings chapter would map from our syntactic language into
  a-mir-formality's input language, which should be straightforward thanks to the all the desugarings;
- The third chapter would contain a-mir-formality itself, and would run its various checks then map
  into MiniRust;
- The final chapter would contain MiniRust itself, and would interpret the program.

Some desugaring steps like method resolution or autoderef need type/trait information which are the
domain of a-mir-formality,
so the integration with a-mir-formality will actually need to be tighter than I just sketched.
Figuring out how exactly that would look like is part of the work to be done in this goal period.

#### Guarantees and Evolution

An important aspect of this vision is that I'd want the spec to have clearly delineated "authoritative"
and "non-authoritative" sections.
The authoritative sections would contain the things we intend to be stable guarantees,
or that already are as per the Reference.
The non-authoritative sections would implement something reasonable in cases
where the Reference does not say much and we don't feel ready to propose something.
We will warn users when their program exercises a non-authoritative code path.

This makes it possible for the spec to be executable without setting everything in stone.

On top of that, I'd like the spec to have a "feature gate" system
to make it possible to experiment with new features.

#### Non-Goals

The spec will not cover the standard library beyond builtin language items.

The spec will not cover low-level details like FFI and compiler flags
beyond what's relevant to the surface language.

The spec will not try to emit nice error messages on invalid user code,
unless we can do so without making the implementation much noisier.

#### Demo

I've started working on the first chapter to get a sense of how that might work,
you can see it [in the
book](https://nadrieril.github.io/rust-via-desugarings/language/overview.md.html).
I adapted the grammar definitions from the Reference to make them into a real parser.
See for example the section that defines
[functions](https://nadrieril.github.io/rust-via-desugarings/language/items/functions.md.html).

[Later on](https://nadrieril.github.io/rust-via-desugarings/pipeline/funsig.md.html)
you can see the implementation of some baby desugarings.
The real thing should be better structured and with better English text.

LLM disclaimer: I let an LLM write the tooling for that demo (e.g. the mdbook preprocessors that
display the grammar nicely and the code that translates the grammar into something a parser
generator can read). I also used one to generate the mapping into MiniRust to make it possible to
run some small programs; this is obviously temporary.
The rest of the book is carefully human-crafted.
See [the intro](https://nadrieril.github.io/rust-via-desugarings/introduction.html) for more
details on LLM use.

### Work items over the next year

For this goal period the objective
will be to put the overall structure into place,
most importantly figuring out
integration with a-mir-formality/MiniRust.

Then we want to have enough structure in place to be able to end-to-end evaluate some real programs:
parse, desugar, and interpret them.
The spec would have a test suite that compares its behavior with rustc.

Beyond that, we would like to fully flesh out a feature or two.
Candidates include method resolution and autoderef, as these
involve trait resolution,
as well as pattern-matching and temporary lifetime extension,
as these are complex yet self-contained.

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Parse functions and traits | Nadri    |       |
| Run rustc on the test suite to compare | Nadri    |       |
| Compile to a-mir-formality's expression language | Nadri    |       |
| Hook into a-mir-formality for trait solving | Nadri    |       |
| Implement temporary lifetime extension | Dianne or Nadri    |       |
| Implement method resolution | Nadri    |       |
| Implement some other syntactic feature | ?    | Optional |

## Team asks

| Team        | Support level | Notes                                   |
| ----------  | ------------- | --------------------------------------- |
| [lang]      | Small         | Ensure buy-in & alignment with this team |
| [opsem]     | Small         | Ensure buy-in & alignment with this team |
| [types]     | Small         | Ensure buy-in & alignment with this team |
| [formality] | Medium        | We'll need help to usefully integrate with formality |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor (1 year, full-time) | Ask @dianne | No | |

## Frequently asked questions

### What's the link with the [formal spec vision you blogged about](https://nadrieril.github.io/blog/2026/06/16/formal-spec-vision.html)?

I do want to clarify this.
That article describes a specific vision I have for the final spec,
which includes opinions about how I'd like us to formalize trait solving and other aspects of our
static semantics.

This mixes two things: the first is this here project of making an executable spec
based on desugarings, the second is a distinct [idea](https://goals.rust-lang.org/2026/dictionary-passing-style-experiment.html)
about how we might go about formalizing trait solving.

The two are rather independent, and this here proposal leaves all questions of specifying static
semantics to a-mir-formality/t-types.

### Does this require new language guarantees to even work at all?

Hopefully, not so much.

The thing we can't avoid is MiniRust: it is needed to run any program at all,
so the implicit basis of this work will be that MiniRust is authoritative,
and our mapping into it will have to be part of that authoritativeness.

Beyond that, we have two tools in our belt: non-determinism, and English text.
If a feature is underspecified, e.g. the order in which patterns are matched isn't guaranteed,
we can make the implementation non-deterministically pick an arbitrary order.
If that's not sufficient, we can write English text like "some unspecified procedure transforms
patterns into simple `if` conditions that involve the place being matched on".
Then we implement something reasonable in the non-authoritative section of the spec.

In both cases an actual execution of the spec may display non-guaranteed behavior
(and we'll warn the user when this occurs),
but at least we can always have _something_ in the authoritative section.

### Does this approach close any doors?

I honestly don't know.

I do expect that if we make something like this authoritative,
it will influence how we design features and what kind of stable guarantees
we can/want to give.
We may even be blinded by implementability and fail to consider some features because
they don't map nicely into how the spec is structured at the time.

But I don't think there's any feature we *couldn't* reasonably specify in this programmatic way,
for the simple reason that it must be implementable in `rustc` anyway.


[^1]: The "after" would of course be automatically computed by running the parser and the step in
question :3

[a-mir-formality]: https://github.com/rust-lang/a-mir-formality
[MiniRust]: https://github.com/minirust/minirust
[specr]: https://github.com/minirust/specr
[rust-via-desugarings]: https://nadrieril.github.io/rust-via-desugarings/
