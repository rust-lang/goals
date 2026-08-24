# Evaluate approaches for seamless interop between C++ and Rust

| Metadata           |                                    |
| :----------------- | ------------------------------     |
| Point of contact   | @tmandry                           |
| Status             | Accepted                           |
| Zulip channel      | N/A                                |
| Tracking issue     | [rust-lang/goals#253] |

## Summary

Seriously consider what it will take to enable Rust adoption in projects that must make use of large, rich C++ APIs. Map out the space of long-term solutions we are interested in. These solutions should enable interop between Rust and other languages in the future.

## Motivation

Rust has seen broad and growing adoption across the software industry. This has repeatedly demonstrated the value of its commitment to safety and reliability. Memory safety, in particular, has caught the notice of governmental bodies in the [European Union][cra] and the [United States][wh], among others.

We should aim to spread the benefits of Rust and its underlying ideas as far as possible across our industry and its users. While the current uptake of Rust is encouraging, it is limited today to areas where Rust adoption is relatively easy. There exists a large portion of production code in use today that cannot feasibly adopt Rust, and it is time we looked seriously at what it would take to change that.

[cra]: https://digital-strategy.ec.europa.eu/en/library/cyber-resilience-act
[wh]: https://bidenwhitehouse.archives.gov/oncd/briefing-room/2024/02/26/press-release-technical-report/

### The status quo

#### Costs of memory unsafety

Memory safety vulnerabilities are the most costly kinds of vulnerabilities, both for product owners and their users. These vulnerabilities and their costs have persisted despite the deployment of many mitigation measures in memory unsafe languages which often impose costs of their own.[^ag] [^rust-in-android]

Experience has shown that regardless of the size of an existing codebase, incrementally adopting a memory safe language like Rust in new code brings roughly linear benefits in terms of new memory safety vulnerabilities. **This is because most vulnerabilities come from new code, not old code.**[^android] This means Rust adoption has value even if only adopted in new code.

Given the growing recognition of this problem from within various technical communities, major technology companies, and major governmental bodies, there is increasing pressure to adopt memory safe languages across the board for all new code. As this proposal explains, this presents both a significant opportunity and a significant challenge for Rust.

[^ag]: <https://alexgaynor.net/2020/may/27/science-on-memory-unsafety-and-security/>
[^rust-in-android]: <https://security.googleblog.com/2021/04/rust-in-android-platform.html>
[^android]: See <https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html> and <https://security.googleblog.com/2022/12/memory-safe-languages-in-android-13.html>.

#### Obstacles to memory safety

Roughly speaking, there are three axes to adoption of memory safety: Social, Technical, and Economic. Making progress along one axis can overcome blockers in the others.

For example, safety has become more socially desirable in many technical communities over the years, which has led to the development of mitigation measures and the adoption of languages like Rust. This has come partly as a result of the recognition of the economic costs of memory safety vulnerabilities.

For C/C++ this has led to an improvement along the technical front in terms of automated checking, in both static and dynamic tooling. However, this protracted effort has also revealed the limits of such an approach without language changes. While there have been calls for C++ to adopt memory safety features,[^safe-cpp] they have not gained traction within the C++ standards body for a combination of technical, social, and economic reasons.[^corentin-profiles]

[^safe-cpp]: <https://safecpp.org/draft.html>
[^corentin-profiles]: <https://cor3ntin.github.io/posts/profiles>

#### Obstacles to Rust adoption

> Changing languages at a large scale is fearfully expensive.[^oncd]

[^oncd]: <https://downloads.regulations.gov/ONCD-2023-0002-0020/attachment_1.pdf>

Rust itself is a major technical breakthrough that enables safety from all kinds of undefined behavior, including spatial safety, temporal safety, and data race safety, with very high confidence. This makes it appealing for those looking to introduce safety to their codebase. Rust adoption is feasible in the following situations:

##### Feasible: New codebases with Rust-only dependencies

This includes completely new projects as well as complete rewrites of existing projects, when such rewrites are socially and economically viable.

##### Feasible: Interprocess boundaries

Projects with a natural interprocess boundary between components are more easily migrated to Rust. Because of the loose coupling enforced by the boundary, the project can be incrementally migrated one component at a time. Microservice architectures with their RPC/HTTP boundaries are one example of this.

##### Feasible: Small, simple intraprocess API surface

Projects with a small, simple API surface that can be manually expressed in terms of the C ABI. This boundary, expressed and invoked in `unsafe` code, is prone to human error. It can be maintainable when the surface is small enough, but this also means that Rust adoption can *decrease* safety at the language boundary.

##### Feasible: Larger intraprocess API surface, but with limited vocabulary

Projects with a limited API vocabulary are able to use one of the existing interop tools like bindgen, cbindgen, or cxx.

##### Infeasible: Everything else

The fact that all of these options exist and undergo active development is a testament to the value developers see in Rust adoption. However, they leave out a large portion of production use cases today: Projects that make rich use of an API in a language like C++ where comparatively limited interop support exists for Rust, and that link in enough code to make rewriting infeasible.

Furthermore, the limitations of current interop tooling are not simply a matter of adding features. Many of them stem from a mismatch in the expressiveness of the two languages along various axes. As one example, C++ and Java both support overloading while Rust does not. In some cases this mismatch is broadly accepted as a missing feature in Rust that will be added in time. In others, Rust's lack of expressiveness may be considered a feature in itself.

These mismatches point to the limitations of such approaches. If we attempt to solve them one at a time, we may never reach the "shiny future" we are working towards.

### The next 6 months

We do not propose any specific deliverables over the next six months. We only propose a discussion with the Language, Compiler, and Libs teams that takes a serious look at the problem space and what it would take to solve it. This discussion should incorporate lessons from existing projects and lay the foundation for future explorations and engagements.

Possible discussion topics include:

* Coverage of rich C++ APIs, including those that make use of language features like templates, (partial) specialization, and argument-dependent lookup. (Lang + Compiler)
* Seamless use of "vocabulary types" like strings, vectors, and hashmaps, including the various kinds of conversions in source and at the ABI level. (Lang + Libs)
* A standard IDL for describing a Rust API/ABI that can be produced by the Rust compiler. (Lang + Compiler)

### The "shiny future" we are working towards

It is essential that our industry adopts memory safety broadly. To realize this, Rust should be feasible to adopt in any application, particularly those which prioritize performance and reliability in addition to safety.

This includes making Rust feasible to adopt in both new and existing applications that make rich use of APIs in memory unsafe languages like C++. To the extent possible, incremental Rust adoption should only *increase* safety, never *decrease* it.

Given that this is a highly ambitious, multi-year project, we should begin with presenting the problem space as accurately as possible to the Rust language team as a way to receive guidance and build alignment on overall direction.

## Design axioms

This goal adheres to the general design axioms in the interop initiative's [problem statement](https://github.com/rustfoundation/interop-initiative/blob/main/problem-statement.md#the-goals):

* Build the foundations for a better future while actively improving the present
* Pursue high-quality interoperation from both sides
* Pursue general-purpose interoperability (not tied to a specific toolchain/IR)
* Avoid changes to Rust itself that would undermine its core values
* Only change the language or standard library where external library solutions are insufficient

In addition, it proposes the following axioms:

* Seek solutions that make 100% coverage possible. This means 100% of functions and methods defined in one language are callable in the other language. This may require some APIs to be unergonomic and/or unsafe to call.
* Minimize the potential for human error. Interop should leverage trusted, automated tooling wherever possible.
* Extend contracts between languages where possible. For example, a strongly typed interface in one language should be equally strongly typed in the other language, subject to the constraints imposed by that language.
* Introduce zero overhead when calling between languages.
* Prefer solutions that are general enough to apply to languages beyond C++.

## Ownership and team asks

**Owner:** @baumanj and @tmandry

| Task                         | Owner(s) or team(s)                    | Notes                                   |
|------------------------------|----------------------------------------|-----------------------------------------|
| Discussion and moral support | ![Team][] [lang], [compiler], [libs]     |                                         |
| Design meeting               | ![Team][] [lang], [compiler], [libs]     | 2-3 meetings expected; all involve lang |
| Author design doc            | @tmandry                               |                                         |
| Author design doc            | ![Help wanted][]                       |                                         |
| Author design doc            | ![Help wanted][]                       |                                         |

## Frequently asked questions

None yet.
