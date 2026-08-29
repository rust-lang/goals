# Borrow checking in a-mir-formality

| Metadata         |                                                                                  |
|:-----------------|----------------------------------------------------------------------------------|
| Contact | @nikomatsakis                                                                    |
| Status           | Proposed                                                                         |
| Tracking issue   | [rust-lang/goals#122]                                               |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |
| [types] champion | @nikomatsakis |

## Summary

Extend a-mir-formality with the ability to represent function bodies as MiniRust programs; a type checker based on the MIR type checker from the compiler; and a model that covers key parts of the the Polonius Alpha proposal.

## Motivation

The goal of the a-mir-formality project is to (a) share and capture knowledge of how Rust's static type checking works, including the type checker, trait checker, borrow checker, and in-progress areas like const generics; (b) enable prototyping and exploration of new ideas in a lighterweight environment that still exposes the "inherent complexity" of the feature while avoiding the "accidental complexity" of building a full-featured implementation in the compiler; (c) be a middle ground that can help us validate and prove Rust's type safety. Achieving that last bullet requires integrating with MiniRust, which models the behavior of a Rust program when executed ("operational semantics") and provides rules for what constitutes undefined behavior. If we align these two models, we should be able to test with fuzzing that, e.g., no fully safe program that passes the type check can, when executed with MiniRust, cause undefined behavior.

### The status quo

Currently, a-mir-formality provides a partial formalization of Rust's type and trait system, but it has significant limitations:

1. **No function body representation**: The current model doesn't include function bodies at all, focusing primarily on types and traits. This prevents modeling program execution and control flow.

2. **Absence of type checker**: a-mir-formality doesn't yet include an implementation of a type checker, which is essential for validating that programs conform to Rust's type rules.

3. **No borrow checking model**: The model doesn't include any representation of Rust's borrow checking system, including the newer Polonius approach.

4. **Gaps in the type system model**: While the model covers key parts of Rust's type and trait system, there are still notable gaps that limit its completeness.

5. **Limited knowledge sharing**: Knowledge and understanding of a-mir-formality are fairly limited. We need more people to familiarize themselves with the setup to broaden expertise and ensure sustainability.

These limitations affect several key stakeholders:

- **Rust language designers** who need a lightweight environment to prototype and explore new ideas without the accidental complexity of the full compiler
- **Compiler contributors** who would benefit from shared, accessible knowledge about how Rust's static type checking works
- **Researchers and educators** who need formal models to validate Rust's type safety claims and teach its concepts
- **Tool developers** who could leverage formal models to build more accurate analysis tools

### The next 6 months

Over the next 6 months, we will focus on three key extensions to a-mir-formality:

1. **MiniRust Integration**: Implement the ability to represent function bodies as MiniRust programs, creating a bridge between static and dynamic semantics.

2. **MIR Type Checker**: Develop a type checker based on the MIR type checker from the compiler, ensuring alignment between the formal model and actual implementation.

3. **Polonius Alpha Model**: Create a model covering key parts of the Polonius Alpha proposal, advancing our ability to formalize and reason about Rust's borrow checking.

### The "shiny future" we are working towards

The long-term vision for a-mir-formality is to develop a comprehensive formal model covering the key components of Rust's type system, trait resolution, and borrow checking. This model will evolve alongside the language, being maintained as new features are added to Rust and serving as a platform for prototyping language extensions. Additionally, a-mir-formality will integrate with complementary models like MiniRust and the Rust specification, creating a foundation for validating Rust's type safety guarantees and sharing knowledge about the language's semantics.

## Design axioms

The goal of a-mir-formality is to be a step in between a high-level, mathematical description of Rust's system (as one might find when doing proofs) and the full details of the compiler (which include significant additional complexity for performance, diagnostics, etc). It should be reasonable to assert that if a-mir-formality is sound, then the compiler is sound modulo small implementation bugs.

* **Recognizably the compiler.** The goal of a-mir-formality is that the algorithms can be mapped to the compiler's algorithms in a fairly straightforward way (though the compiler will have additional optimizations, caching, and all kinds of concerns that don't matter to a-mir-formality).
* **Capture what's essential.** a-mir-formality doesn't need to cover every detail of Rust but it should capture the ones that are important when reasoning about soundness and safety or the type system.
* **The compiler should be a sound, but incomplete version of a-mir-formality.** Given some program P that falls within the scope of what a-mir-formality models, if the compiler accepts P, then a-mir-formality should accept P. But there may be some programs that the compiler refuses to accept because of implementation limitations that a-mir-formality would accept because it is willing to (for example) explore a larger search space.

[da]: ../about/design_axioms.md

## Ownership and team asks

This section outlines the work to be done and the specific asks from Rust teams. A key challenge we face is limited knowledge and understanding of a-mir-formality's setup. To address this, we're asking for standard reviews from types team members and dedicated review for Polonius-related work.

| Task                                      | Owner(s) or team(s)                | Notes |
|-------------------------------------------|-----------------------------------|-------|
| Implementation of MiniRust integration    | @tiif                             | Extending a-mir-formality to represent function bodies as MiniRust programs |
| Implementation of MIR type checker        | @tiif                             | Creating a type checker based on the compiler's MIR type checker |
| Implementation of Polonius Alpha model    | @tiif                             | Modeling key parts of the Polonius Alpha proposal |
| Standard reviews                          | ![Team][] [types]                 | Help familiarize more people with the a-mir-formality setup through PR reviews |
| Dedicated reviewer                        | ![Team][] [types]                 | Assign specific reviewers for Polonius Alpha model implementation (@lqd) |
| Mentorship                                | @nikomatsakis                     | Providing guidance and mentorship for the implementation work |

### Definitions

For definitions for terms used above, see the [About > Team Asks](https://goals.rust-lang.org/about/team_asks.html) page.

* *Standard reviews* refers to reviews for PRs against the repository; these PRs are not expected to be unduly large or complicated.
* *Dedicated review* means identifying an individual (or group of individuals) who will review the changes, as they're expected to require significant context.

## Frequently asked questions

### What is a-mir-formality?

a-mir-formality is a formal model of Rust's type system, available at [github.com/rust-lang/a-mir-formality/](https://github.com/rust-lang/a-mir-formality/). It aims to provide a rigorous mathematical foundation for understanding Rust's type system and semantics.

### What is MiniRust?

MiniRust is a model of the behavior of a Rust program when executed ("operational semantics") and provides rules for what constitutes undefined behavior. Integrating a-mir-formality with MiniRust will allow us to verify that programs that pass the type check don't cause undefined behavior when executed.
