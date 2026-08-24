# C++/Rust Interop Problem Space Mapping

| Metadata            |                                    |
|:--------------------|:-----------------------------------|
| Point of contact    | @teor2345                          |
| Status              | Accepted                           |
| Tracking issue      | [rust-lang/goals#388] |
| Zulip channel       | [t-lang/interop][channel]          |
| [compiler] champion | @oli-obk                           |
| [lang] champion     | @tmandry                           |
| [libs] champion     | @dtolnay                           |

[channel]: https://rust-lang.zulipchat.com/#narrow/channel/427678-t-lang.2Finterop

## Summary

Document a set of technical issues related to C++/Rust interoperability with broad community consensus to serve as a starting point for proposed solutions and facilitating cooperation among stakeholders in both language communities.

## Motivation

C++ has been a leading systems language for most of its 40 year history. There are billions of lines of code representing trillions of dollars of value stored in C++ codebases. In domains where systems languages are required, there are very few options and even fewer with the safety properties of Rust. In the near term, it is neither feasible, nor advisable to rewrite a significant fraction of the C++ actively used today. [Empirical studies] have shown significantly more benefit in writing *new* code in memory-safe languages. This makes enabling Rust development within C++ codebases essential, and doing so requires high-performance, safe, ergonomic interop.

There are many challenges to this kind of interop, and they can be solved in many different ways. To the extent that discrete problems within the interop space can be solved in a shared way, both communities stand to benefit, but thus far there has been relatively little progress in implementing solutions other than external libraries, each of which typically have their own approach, requiring significant investment for end users. To build better solutions to foundational interop problems that may require changes to the standard libraries, toolchains or the languages themselves, we will first clearly and rigorously define the individual problems and solution requirements with input from experts from both languages including implementors of compilers and standard libraries. Achieving consensus in defining the problems will facilitate the creation and discussion of proposed solutions with the appropriate stakeholders and improve the chances that solutions succeed in broadly benefiting interop consumers.

[Empirical studies]: https://www.usenix.org/conference/usenixsecurity22/presentation/alexopoulos

### The status quo

C++/Rust interoperation is an incredibly broad problem space owing to the fact that the C++ language has been continuously developed for 40 years and has actively maintained implementations on numerous architectures and [several different toolchains]. Furthermore, the requirements of interop consumers varies according to the application as well as constraints on compilers and C++ language versions. Neither Rust nor C++ has language- or standard library-level support for direct interoperation. Instead, the C system ABI is used as the *lingua franca* for directly calling into the opposite language. While there are ways to achieve cross-language interoperation an a language-agnostic fashion, such as interprocess communication, this proposal is concerned with high-performance, FFI-based interoperation. This is required in performance-critical domains where Rust and C++ are among the few viable language choices and the problems for this kind of interoperation may benefit from coordinated changes in both languages. For more on the domain of interest, see the [C++/Rust Interoperability Problem Statement].

Currently there are a variety of approaches to this kind of interop from tools like [bindgen] and [cbindgen] which automate only the creation of type and function declarations for FFI calls, to higher-level approaches like [cxx], [crubit] and [zngur], which provide varying levels of automation to handle both binding generation and sharing of data across the FFI boundary. All differ in the specifics and though there are many elements to their approaches which are similar, they do not share any common foundation and each builds a solution supported only by the very limited facilities of Rust and C++ to declare functions and data which are compatible with the C system ABI.

Different use cases and priorities account for part of the reason there are a variety of tools. For interop that is limited to well-defined module boundaries and can exploit existing C API which are not too large, the bindgens often suffice, but sacrifice safety and ergonomics. Tools like cxx and zngur provide higher levels of abstraction and safety, but require the user to define an IDL-like interface boundary. Finally, crubit strives to be a universal tool and automatically expose *all* interfaces to the opposite language which entails considerably more complexity and deep integration with the build system, so it is not yet viable for general purpose use.

In the first year of the C++/Rust Interoperability Initiative, I've engaged with many of the parties interested in interop and facilitated direct interactions with members of the Rust Project, [WG21] (the C++ ISO standardization committee) and several large tech organizations. I believe for the foreseeable future, there will be multiple approaches to interop, but owing to the very primitive language-level support all these tools are built upon, there is a significant space of shared problems which can benefit from collaboration and shared solutions. Especially when it comes to changes at the language-level, there is a heightened standard for broad utility, and coordinating work across languages with very different evolution processes will require a clear understanding of the specific problems and expert insight regarding the feasibility and impact of changes to the standard libraries, toolchains and core languages. As a member of the Rust Foundation, and a representative to WG21, I'm well-positioned to coordinate these efforts. Having seen first-hand the difficulty in agreeing on solutions and the overwhelmingly large subject area required to understand them, I think the next step is to focus on clearly defining and documenting the problems themselves. This will help identify the areas of potential cooperation, facilitate input from interested parties who may otherwise lack the full context, and sets the stage for solutions to be proposed and refined into concrete, cooperative work that may occur in a multitude of venues, including the Rust Project, WG21, LLVM or elsewhere.

[several different toolchains]: https://en.cppreference.com/w/cpp/compiler_support.html
[C++/Rust Interoperability Problem Statement]: https://github.com/rustfoundation/interop-initiative/blob/main/problem-statement.md
[bindgen]: https://crates.io/crates/bindgen
[cbindgen]: https://crates.io/crates/cbindgen
[cxx]: https://crates.io/crates/cxx
[crubit]: https://github.com/google/crubit
[zngur]: https://github.com/HKalbasi/zngur
[WG21]: https://en.wikipedia.org/wiki/ISO/IEC_JTC_1/SC_22

### What we propose to do about it

There has already been a significant discourse on the problems related to interop, but it remains diffused across the web, Zulip and other less-accessible media. Furthermore, the vast majority of this material is in the form of advocating for one tool or approach and typically comes from the perspective of a particular user or organization with specific needs and interests. During this goal period, I will organize as much of it as I can into discrete problem statements which will describe the challenges, prior attempts and current approaches and desired solution properties, but without proposing specific solutions. These will be collected under the interop initiative repo, and I will both accept and solicit input from relevant experts. The goal is achieving consensus around what the problems themselves are as objectively as possible, so parties interested in proposing, reviewing or implementing solutions can organize around the issues of relevant interest and expertise and optimize for shared solutions.

Looking further ahead, a well-defined map of the discrete problems in the interop space will allow for the creation of discrete solution proposals, which can be refined and debated by interested members of both the Rust and C++ communities and eventually turned into Rust RFCs, WG21 papers, change processes targeted at other relevant communities such as LLVM, or direct code contributions. There are already many such items of work in process such as [arbitrary self types], [pin ergonomics] or [trivial relocatability], and in the shiny future, the problem space map would elucidate the connection between such work and the problems it addresses. This would serve as the central point of collaboration between the two communities to understand the interop-related challenges and work together efficiently. Interested parties can find the problems that are relevant to them (or help define them if they are missing) and then…

* find the shared problems and work with other interested parties towards a shared solution
* accept with clear reasoning when different solutions are required for a shared problem
* not waste time engaging about problems that aren't relevant to them

Given the large investment and long time frames involved in language-level changes, it's particular valuable to have a solid grounding in the problem itself. This space could also grow into a shared community to help determine and facilitate the process of implementing solutions, whether they be creating RFCs, WG21 proposals or something else.

[pin ergonomics]: https://github.com/rust-lang/rust/issues/130494
[arbitrary self types]: https://github.com/rust-lang/rust/issues/44874
[trivial relocatability]: https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2025/p2786r13.html

## Design axioms

Problem statements should…
* define the impact, consequences and acceptance criteria for solutions
* reference prior solution work, but without proposing or analyzing solutions
* be based objective facts supported by expert opinions and generally strive to be informative and uncontroversial
* provide a high-level explanation for engineers who aren't experts in the cause, but are likely to experience the impact
* provide references to additional background information

## Work items over the next year

The output will be an interop initiative repository containing discrete problem statements.

| Task                      | Owner(s) | Notes |
|---------------------------|----------|-------|
| Author problem statements | @teor2345|       |


## Team asks

Fundamentally, this goal is about organizing knowledge in a useful way. I will be primarily responsible for collecting and structuring it, but do not intend to do original analysis since there are greater experts to defer to. The most valuable participation from Rust teams will be in reviewing the accuracy of the collected information. I do not expect the burden to be great since the individual problems are intended to be narrowly scoped and reviewers will be experts in the domain. Demands will depend on the specific problems documented and the teams that are deemed relevant

| Team       | Support level | Notes                    |
|------------|---------------|--------------------------|
| [compiler] | Small         | Reviews                  |
| [lang]     | Small         | Reviews                  |
| [libs]     | Small         | Reviews                  |
| [opsem]    | Small         | Problem statement review |


## Frequently asked questions

### Is this intended to be a living document or a snapshot in time?

A living document. It's unlikely to ever be complete, especially considering both Rust and C++ continue to change, but since one of the main points of the document is to attract more input and inspire new solution proposals, openness to change is a feature.

### Do you want to be the curator of all content or go for more of a wiki approach?

At least initially (and almost certainly for the lifetime of this goal), I plan to be the sole curator, but I will actively encourage and solicit contributions to ensure the content is as accurate and objective as possible. Over time, additional curators may be added, and at some point it may make sense to transition to a more wiki-like approach if curation becomes a bottleneck, but I can't say for sure.

### How should the problems be prioritized?

At the very beginning, the goal will be breadth: attempt to give a name and basic description to as many of the relevant problems as possible. This will form a sort of skeleton to aid with discoverability and motivate contribution from relevant experts. From there, priority will be given to collecting information that already exists spread across various sources such as meeting notes, conference talks, forum posts and Zulip streams. Problems which are causing people current pain and which have competing solution ideas will be especially prioritized to encourage them to be formalized in response to a common understanding of the problem. Finally, problems which are most likely to have solutions requiring longer-term processes such as RFCs and WG21 papers will be prioritized to get things moving sooner and problems with uncontroversial solutions will be prioritized because we all could use some easy wins.

### If you had unlimited time and resources, wouldn't it be advisable to rewrite all the C++ in Rust?

I don't think it's clear that it is! While the risk of UB goes away if you can rewrite it all in *safe* Rust, there are lots of other kinds of bugs that can occur, and they're far more likely to occur in new code, so the older and more real-world use code has, the less likely it is to contain bugs, regardless of language.

### I notice you wrote "implementors" above. Why not "implementers"?

This bothered me too, and [this internals thread](https://internals.rust-lang.org/t/spelling-bikeshed-implementor-or-implementer/16926/14) explained why it was so noticeable to me (and likely other Rustaceans).
