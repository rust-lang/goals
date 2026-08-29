# Rust Stabilization of MemorySanitizer and ThreadSanitizer Support

| Metadata         |                                    |
| :--              | :--                                |
| Contact | @jakos-sec                         |
| Status           | Proposed                           |
| flagship         | Flexible, fast(er) builds          |
| Tracking issue   | [rust-lang/goals#403] |
| Zulip channel    | N/A                                |


## Summary

Stabilize the MemorySanitizer and ThreadSanitizer support. This includes fixing open bugs for the sanitizers to open a path for stabilization and the necessary infrastructure changes to provide precompiled and instrumented standard libraries for the sanitizers.

## Motivation

Sanitizers help with increasing the robustness and security of software and have been increasingly adopted as part of the software development life cycle. Even though the Rust programming language provides memory and thread safety guarantees, use of Unsafe Rust and foreign code in mixed-language binaries do not provide the same memory and thread safety guarantees. Thus, support for sanitizers must be added to the Rust compiler for secure Rust adoption ([source](https://hackmd.io/@rcvalle/S1Ou9K6H6#Organize-and-stabilize-support-for-sanitizers)).

In order for them to be properly usable, the sanitizer support should be stabilized so it is no longer required to use a nightly toolchain and build your own standard libraries.

### The status quo

Currently, there is unstable support for several sanitizers (address, hwaddress, memtag, memory, thread, leak, cfi, kcfi, safestack, shadow-call-stack). The AddressSanitizer and LeakSanitizer (that do not require rebuilding an instrumented standard library) are close to being stabilized (https://github.com/rust-lang/rust/pull/123617). The others require more work in order to become stabilized.

### The next 6 months

The goal is to stabilize MemorySanitizer and ThreadSanitizer for [Tier 1](https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-1-with-host-tools) targets and provide a way to use them without rebuilding the standard library (which currently is also behind an unstable flag). This requires a way to ship sanitizer instrumented standard libraries (for Memory- and ThreadSanitizer) through [rustup](https://rustup.rs/).

### The "shiny future" we are working towards

All sanitizers (including memtag, cfi, kcfi, safestack, shadow-call-stack) are fully stabilized and supported on all targets supported by the Rust compiler.
In the meantime we work towards at least supporting [Tier 1](https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-1-with-host-tools) targets for all of them.

## Ownership and team asks

**Owner:** [@jakos-sec](https://github.com/jakos-sec), [@1c3t3a](https://github.com/1c3t3a)

| Task                         | Owner(s) or team(s)                                                              | Notes         |
| ---------------------------- | -------------------------------------------------------------------------------- | ------------- |
| Discussion and moral support | ![Team][] [compiler], [infra]                                                    |               |
| Implementation               | [@jakos-sec](https://github.com/jakos-sec), [@1c3t3a](https://github.com/1c3t3a) |               |
| Stabilization decision       | ![Team][] [compiler]                                                             |               |
| Standard reviews             | ![Team][] [compiler]                                                             |               |
| Dedicated reviewer           | ![Team][] [bootstrap], [project-exploit-mitigations]                                 |               |

### Definitions

For definitions for terms used above, see the [About > Team Asks](https://goals.rust-lang.org/about/team_asks.html) page.

* *Discussion and moral support* is the lowest level offering, basically committing the team to nothing but good vibes and general support for this endeavor.
* *Author RFC* and *Implementation* means actually writing the code, document, whatever.
* *Design meeting* means holding a synchronous meeting to review a proposal and provide feedback (no decision expected).
* *RFC decisions* means reviewing an RFC and deciding whether to accept.
* *Org decisions* means reaching a decision on an organizational or policy matter.
* *Secondary review* of an RFC means that the team is "tangentially" involved in the RFC and should be expected to briefly review.
* *Stabilizations* means reviewing a stabilization and report and deciding whether to stabilize.
* *Standard reviews* refers to reviews for PRs against the repository; these PRs are not expected to be unduly large or complicated.
* *Prioritized nominations* refers to prioritized lang-team response to nominated issues, with the expectation that there will be *some* response from the next weekly triage meeting.
* *Dedicated review* means identifying an individual (or group of individuals) who will review the changes, as they're expected to require significant context.
* Other kinds of decisions:
    * [Lang team experiments](https://lang-team.rust-lang.org/how_to/experiment.html) are used to add nightly features that do not yet have an RFC. They are limited to trusted contributors and are used to resolve design details such that an RFC can be written.
    * Compiler [Major Change Proposal (MCP)](https://forge.rust-lang.org/compiler/mcp.html) is used to propose a 'larger than average' change and get feedback from the compiler team.
    * Library [API Change Proposal (ACP)](https://std-dev-guide.rust-lang.org/development/feature-lifecycle.html) describes a change to the standard library.

## Frequently asked questions

None yet.
