# Target directory size reduction

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Contact | @jyn514                                   |
| Status           | Proposed                                                                         |
| What and why     | Reduce the size of temporary compilation artifacts so developers need less disk space |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |
| [cargo] champion | @ranger-ross |
| [compiler] champion | @petrochenkov |
| [wg-compiler-performance] | @Kobzol |
| [infra] | @Kobzol |

## Summary

Investigate, triage, and decrease the size of intermediate compilation directories (e.g. `target/`).
With a funded team of 3-5 engineers, we aim over the next year to decrease the size of the directory by 60% or more for fresh builds and by 80% or more for repeated builds across version branches.
This includes both decreases in artifact size and garbage-collection within a target directory, such as artifacts from an earlier edit.
We will benchmark based on real crates in the Rust ecosystem, focusing on disproportionately large and widely used crates.
After benchmarking, we will work on multi-pronged approaches that allow parallel work on high-impact interventions.
Our work will be incremental, in the sense that partial work outputs will still be useful; we do not need a full year to start seeing improvements.

Note that this goal does *not* include garbage-collection across directories, such as deleting `cargo -Z script` artifacts.
There is [parallel work][cargo#17524] on that, but it is not currently part of this goal.

[cargo#17524]: https://github.com/rust-lang/cargo/pull/17524

## Motivation

### Why it matters

Disk space is documented as a repeated concern;
the [2025 "State of Rust Survey"][2025-survey] shows it as the second-most common complaint about the Rust toolchain,
second only to slow compilation.

[2025-survey]: https://blog.rust-lang.org/2026/03/02/2025-State-Of-Rust-Survey-results/#challenges-and-wishes-about-rust

We expect reducing disk space to benefit all environments that build Rust programs, but especially:
- developers who heavily use multiple git worktrees/jj workspaces, such as AI-first workflows or contributors to the Rust compiler itself;
- CI jobs that cache intermediate artifacts, both in storage space and in upload/download speeds;
- persistent remote caches, such as used by Bazel and Buck2, primarily in upload/download speeds and disk IO usage.

We also theorize that by reducing disk IO, we can speed up wall time for compilation itself.
If our efforts are sufficiently fruitful, we could further decrease compilation time by allowing target directories to fit on a RAM disk (sometimes known as `tmpfs`).

Note that disk space at *compile time* (of intermediate artifacts) is different than disk space at *runtime* (of final artifacts).
The [binary size reduction] roadmap tracks final artifacts; this goal is focused on intermediate artifacts.
We expect that we may need to coordinate between teams, but that our efforts will not substantially overlap.

[Binary size reduction]: https://goals.rust-lang.org/2026/roadmap-binary-size-reduction.html

### The status quo

Compiling Rust programs takes three primary resources:
1. CPU time and wall time
2. Memory use
3. Disk space

CPU time, and to some extent wall time and memory use, has been relentlessly optimized and micro-optimized for many years in Rustc.
Target directory sizes have had various ideas suggested on the Cargo, but not as much ongoing work due to funding limitaions, and hardly anything has been done on the Rustc side.

I am aware of the following ongoing work:
- The [cross workspace cache] project goal, which reduces the number of total artifacts, but does not reduce the size of each artifact.
- [Deduplicate build artifacts across workspaces][cargo#17453] again reduces the number of total artifacts.
- Garbage-collection work to reclaim space between builds has several tracking issues ([cargo#5026], [cargo#13136], [cargo#13060]), but no project goal.
- Reusing caching between `check` and `build` is being investigated in the [incremental system redesign] goal, but cannot help with initial full builds.
- [`-Zembed-metadata=no`][embed-metadata] avoids creating unnecessary metadata sections, but does not shrink the sections themselves when they're created.
- [Reduce debuginfo to `line-tables-only` in the `dev` profile][cargo#17518] reduces the size of debuginfo in the most common scenarios, but does not improve builds that have full debuginfo.

Furthermore, the Cargo team has suggested several possible improvements to disk size which could be incorporated into the project goal:
- [Reduce how frequently build scripts need to be written][cargo#14948]
- [Give build scripts a dedicated scratchpad for temporary artifacts][scratchpad]
- [Convert build scripts to artifact dependencies][cargo#14903]
- [Pipeline build scripts, not just library crates](https://rust-lang.zulipchat.com/#narrow/channel/628857-t-cargo.2Fbuild-script/topic/Pipelined.20builds/with/620427777)

[cargo#14948]: https://github.com/rust-lang/cargo/issues/14948
[embed-metadata]: https://github.com/rust-lang/rust/issues/139165
[cross workspace cache]: https://goals.rust-lang.org/2026/cargo-cross-workspace-cache.html
[cargo#5026]: https://github.com/rust-lang/cargo/issues/5026
[cargo#13060]: https://github.com/rust-lang/cargo/issues/13060
[cargo#13136]: https://github.com/rust-lang/cargo/issues/13136
[cargo#14903]: https://github.com/rust-lang/cargo/issues/14903
[cargo#17453]: https://github.com/rust-lang/cargo/issues/17453
[cargo#17518]: https://github.com/rust-lang/cargo/pull/17518
[incremental system redesign]: https://goals.rust-lang.org/2026/incremental-system-rethought.html
[scratchpad]: https://rust-lang.zulipchat.com/#narrow/channel/628857-t-cargo.2Fbuild-script/topic/Providing.20a.20dedicated.20scratchpad.2C.20instead.20of.20using.20.60OUT_DIR.60

### What we propose to do about it

We believe the primary obstacles to shrinking intermediate artifacts are that it's currently difficult to measure sizes, and a lack of funding for the work.
The closest tools are `llvm-size` to show overall section sizes; `rustc -Z meta-stats` to show `.rmeta` contents; and `size:query_cache` on [perf.rust-lang.org][prlo], but none give a breakdown of `.rmeta` or incremental files by query, and are difficult to correlate across invocations.

[prlo]: https://perf.rust-lang.org

At a high level, our plan is:
1. Benchmark what takes up the space, and include those benchmarks in `rustc-perf` so they are triaged alongside other performance improvements and regressions.
2. Near-term (1-6 months), use those measurements to guide simple improvements, with a focus on "easy wins" that greatly reduce size without much implementation work.
3. Mid-term (3-12 months), use measurements to guide structural improvements that need design work or extensive review.
4. Long-term (6-18 months), make large refactors, and change best-practices in the ecosystem to avoid disk usage.

We will document our approach and tooling so that sizes can be further reduced even after the goal period ends.

We do not believe our long-term goals will be necessary for our proposed 60-80% size improvement in the first year.
We expect mid-term and long-term redesigns to "compose" with the short-term improvements; in other words, major improvements before a redesign will also be major improvements after the redesign.
If we find this not to be the case after experience during implementation, we will revisit our approach.

All structural redesigns will be conditional on benchmarks that show that they're actually a problem.

#### Axioms

- **Avoid an all-or-nothing approach.** Steps can depend on previous steps, but between each step we should have a coherent work product that will be an improvement to the ecosystem even if later steps are never completed.
  Size improvements should stand on their own without requiring a rewrite of subsystems of Rustc or Cargo.
  Only work on sweeping refactors once we have fixed all the "low-hanging fruit".

- **Focus on architectural improvements.** Micro-optimizations look good in the short-term and are hard to maintain in the long-term.
  They can also easily be specific to the precise hardware that was used to run the benchmark.
  By prioritizing algorithmic and architectural improvements, we hope to end up with code that is *more* maintainable than it was to start, while still generating significantly smaller artifacts.

- **Work together.** There are many people who want to work on the compiler and few who have the expertise to do so.
  We have an explicit goal of teaching people about the compiler, especially about the query system and `rustc_metadata`, so that they can continue to maintain these subsystems even after the goal is closed.
  As a corollary, we will choose optimizations that can be worked on in parallel, so that multiple people can be working at once, collaborating and bouncing ideas off of each other.
  *Note*: A common problem when optimizing CPU time is that improvements in one area regress other areas, especially when touching the query system, forcing work to essentially be done in serial.
  We do *not* expect that to be the case here, because so little optimization has been done to date.
  We will benchmark each change to confirm, and will revist our scheduling assumptions every 3 months based on observed conflicts.

- **Use good defaults but allow configuration.** Many software projects are *either* highly configurable (e.g. Kakoune) or work well out of the box (Helix), but few are both. We aim to make improvements that greatly improve disk space without needlessly regressing performance, while still allowing developers to choose higher reductions in exchange for slower compilation time, or vice-versa. For example, environments that already use an in-kernel compressed file system for `target/` may wish to disable debuginfo compression, or remote caching servers that rarely rebuild but have to maintain artifacts indefinitely may wish to prioritize size over speed.

### Work items over the next year

Subgoals (heading level 5) are independent and can be worked on in parallel unless otherwise labeled.
Within a subgoal, tasks are serially dependent unless otherwise labeled.

#### Measurement

##### Benchmark intermediate artifact sizes

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Create a prototype PoC that breaks down `.rmeta` files and `incremental/` directories by size and query across multiple build invocations | @jyn514  |       |
| Extend the PoC to a maintainable tool that can be used outside the project goal team | ? | Likely implemented as a `rustc_driver` or `-Z` flag |
| Add support for running that tool to `rustc-perf` benchmarks | ? | Subgoal id A2. Will automatically give us a graph over time once metrics are hooked up. |
 | (Optional) Add support for auto-triaging whether a size change is statistically relevant | ? | likely not necessary to start since metadata and query cache sizes are stable |

##### Collect representative benchmarks

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Collect a sample of crates that have unusually large artifact sizes | @Zeklandia | Subgoal id B1 |
| Add those crates to the `rustc-perf` benchmark suite | @Zeklandia | Depends on subgoal A2.  May require adding support for chains of dependencies to rustc-perf, since artifact size is impacted by monomorphization |
| Identify the largest sub-sections of `.rmeta`, `.rlib`, and `incremental/` files for those benchmarks | @Zeklandia | Subgoal id B3 |

#### Near-term improvements
These will focus on improving intermediate artifact size for gathered benchmarks.
Each of these can be worked on in parallel.

| Task        | Owner(s) | Notes |
| ----        | -------- | ----- |
| Stabilize and enable `-Z embed-metadata=no` by default | @Kobzol | stablization PR already open but not yet merged |
| Reduce debuginfo in build scripts | @jyn514 | if [cargo#17518] doesn't already apply to build scripts, it could be extended |
| Avoid serializing unnecessary incremental state | ? | needs further investigation |
| Avoid serializing queries on disk where possible. | ? | changes must backed by benchmarks showing that this has little effect on compilation speed |
| Dynamically link the standard library in build scripts | @jyn514 | |

#### Structural improvements

##### Reduce duplicate storage and unnecessary serialized state
Subtasks in this category can be worked on in parallel.

| Task | Owner(s) | Notes | 
| ---- | -------- | ----- |
| Remove duplicate sections between incremental cache and `.rlib` files  | @panstromek | depends on subgoal B3. needs design work. |
| Decrease debuginfo size for .rlib files | @jyn514 | likely through enabling compression; early benchmarks show near-original performance when compressed and unpacked split-dwarf is enabled. needs careful design if binaries are to remain static and portable. may be less urgent once `dev` profiles use `line-tables-only`. |
| Use DWARF type signature computation to avoid duplicating debuginfo in the final binary  | @jyn514 | overlaps with binary-size roadmap, needs coordination |
| Use `dwz` to avoid duplicating debuginfo in the final binary  | @jyn514 | unclear whether this should be Cargo or Rustc's responsibility, needs design work. overlaps with binary-size roadmap |
| Extend Rustc with equivalents of `-gmodules` and `-fno-standalone-debug` to avoid duplicating debuginfo in intermediate artifacts | @jyn514 | large task, needs compiler design work. needs care to avoid making intermediate files non-portable. |

##### Reduce overlap between test binaries
Currently, `cargo test` produces one `libtest` harness binary for each crate in a workspace.
In crates with many workspaces, this leads to a large amount of time spent linking and space taken up by the resulting many binaries.
Can we consolidate these into a single binary, or use dynamic linking to reduce the amount of duplicated space?

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Test on crates with large workspaces to see where the space is taken up | ? |  |
| Investigate possible approaches to see which are feasible | ? | |
| ? | ? | needs design discussion before further subtasks can be created |
##### Decrease build script artifact sizes

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Determine which build scripts generate the largest outputs | @Zeklandia | Depends on subgoal B1 |
| Make targeted improvements to crates in the ecosystem | ? | for example, get build scripts to delete unused temporary artifacts after a successful build |

##### Redesign `incremental/` directories to be GC-able

| Task | Owner(s) | Notes | 
| ---- | -------- | ----- |
| Design discussions with Cargo team | @jyn514 | Most uncertainty. Needs Cargo team capacity. |
| Emit structured info in Rustc that gives Cargo enough info to GC | ? | |
| Extend Cargo to automatically GC incremental directories | ? | |
##### Redesign Cargo's incremental caching for build scripts

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Design discussions with Cargo team | @jyn514 | Most uncertainty. Needs Cargo team capacity. |
| Delete build script executables between reruns | ? | needs care to ensure that outputs can be reused even when the original build script is gone. needs care to ensure that rebuilding a build script doesn't unnecessarily slow down builds.  |

##### Distinguish temporary build script outputs from final outputs

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Design discussions with Cargo team | ? | Most uncertainty. Needs Cargo team capacity. See [existing cargo discussion][scratchpad]. |
| Add Cargo APIs that allow build scripts to communicate this | ? | |
| Extend Cargo to GC temporary build script outputs | ? | |
| Make targeted PRs within the ecosystem to switch crates to use the new APIs | ? | |
 ##### Redesign the `.rmeta`, `.rlib`, and incremental formats for better disk usage.
Improvements to the `Encodable` serialization will result in decreased disk usage across all three file types.

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Design discussions with Compiler team | @panstromek | will benefit from a compiler team sponsor, but not strictly necessary |
| ? | ? | needs design discussion before further subtasks can be created |

##### Garbage collect target directories within a project

| Task         | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Design discussions with Cargo team | @ranger-ross | discussions already ongoing in [cargo#17512], less risk than other design tasks |
| ? | ? | needs design discussion before further subtasks can be created |

## Team asks


| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [cargo]    | Large | We will likely want to substantially redesign how `incremental/` directories and build script outputs are cached. We are confident we can find an approach that works for both t-cargo and t-compiler, and we can be responsible for implementation, but we will need design discussions and review from the Cargo team. |
| [compiler] | Medium | Advice on where to look and which approaches are best; reviews for small-to-medium compiler PRs that improve binary size. We expect efforts to be focused in and around the query system and `rustc_metadata`. We understand that both these areas are quite complicated and fragile; we will clearly distinguish refactors, rearchitectures, and micro-optimizations. Note: Whether this is "medium" or "large" depends on our findings from the PoC. |
| [wg-compiler-performance] | Medium | Assistance selecting benchmarks; reviews for new intermediate artifact measurement support. |
| [bootstrap]    | Small              | We may make small tweaks to bootstrap code or to how `rustc-dev` .rlibs are uploaded. We will endeavour to minimize changes to these areas.                                         |


Suggested reviewers:
- cargo: Ed Page, Ross Sullivan
- compiler: Petrochenkov, Nick Nethercote
- compiler/performance: Jakub Beranek
- infra/bootstrap: Jakub

## Help wanted
Note: at this stage we are primarily looking for contributors who are *new* to this area of the compiler, or new to the Rust project overall, and wish to be mentored as part of this goal.
We have sufficient contributors with existing experience.

|Task                                             |Experience level                                                      |Time investment                                                  |
|-------------------------------------------------|----------------------------------------------------------------------|-----------------------------------------------------------------|
|Create a maintainable benchmarking tool          |Medium Rust experience, compiler experience preferable                |1-3 weeks full-time                                              |
|Interns to improve artifact sizes                |Medium Rust experience, compiler or build system experience preferable|1-12 months part-time or full-time. Multiple positions available.|


## Funding

There are over a dozen workstreams within this proposed goal.
By myself, I cannot implement them all within a year.
Additional funding will allow me to hire and mentor other compiler and build system engineers to split up the work,
not only accelerating timelines but also completing more overall workstreams.
Furthermore, it will allow me to focus on design work rather than implementation, unlocking the larger structural improvements that require coordination between teams.

This goal needs a minimum of (Ask) to run, but for that amount we could only complete 3-5 small workstreams, most likely the items in the "Near-term improvements" section.
Funding up to the full requested amount will allow us to complete all or almost all the proposed workstreams.

I have extensive experience working on the compiler, including the query system and integration work between Rustc and Cargo.
I also have project management experience from my time at Ferrocene,
as well as consensus building experience on policy, design, and implementation work within the Rust project.
As part of this work, I would lead implementation, mentor contributors, and coordinate between the Compiler and Cargo teams.

Ross Sullivan is a current member of the Cargo team and has ongoing work to reduce target directory sizes.
[Early research][cargo#17512] shows the possibility of a 56% or more size reduction in cross-project caches from his ongoing work alone.
Further reductions are possible with more workstreams.
For Ross to join, we would need a substantial amount of the funding to be paid "up-front" at the start of the goal.

Matyáš Racek is a current member of the Compiler Performance working group and has ongoing work to reduce encoding sizes in rmeta files and incremental caches.
He has experience measuring, triaging, and improving the compiler's performance, and his assistance will be invaluable in optimizations within the compiler itself.

[cargo#17512]: https://github.com/rust-lang/cargo/pull/17512
[brlo]: https://blog.rust-lang.org/inside-rust/2026/08/18/reducing-target-dir-size-on-nightly/

Kobzol has been responsible for implementing and stabilizing `-Z embed-metadata=no`, and is a current member of the Council.
`embed-metadata` has reduced the size of release builds by [almost 30%][brlo] in the past, and we are confident that further similar improvements are possible.

Zeklandia has a non-traditional programming background, but has the logistical flexibility to do short-term contract work,
and I hope to mentor her to be a long-term maintainer on these compiler subsystems.


|Purpose                                                                              |Cost                           |Funded  |Sponsor(s)                                   |
|-------------------------------------------------------------------------------------|-------------------------------|--------|---------------------------------------------|
|@jyn514 as owner (6-12 months, full-time)                                            |Ask                            |No      |                                             |
|@ranger-ross as Cargo design contact and implementor (6-12 months, variable)         |Ask                            |No      |                                             |
|@panstromek as wg-perf design contact and implementor (6-12 months, part-time)       |Ask                            |No      |                                             |
|@Kobzol as advisor (6-12 months, part-time)                                          |Ask                            |No      |                                             |
|@Zeklandia as benchmark selector and analyst (variable, part-time)                   |Ask                            |No      |                                             |
|Intern (1-12 months, part- or full-time, multiple openings)                          |TBD                            |No      |                                             |


## Frequently asked questions

TBD

<!-- ### What do I do with this space? -->

<!-- *This is a good place to elaborate on your reasoning above -- for example, why did you put the design axioms in the order that you did? It's also a good place to put the answers to any questions that come up during discussion. The expectation is that this FAQ section will grow as the goal is discussed and eventually should contain a complete summary of the points raised along the way.* -->
