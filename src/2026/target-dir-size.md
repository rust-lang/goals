# Target directory size reduction

| Metadata         |                                                                                  |
| :--------------- | -------------------------------------------------------------------------------- |
| Contact | @jyn514                                   |
| Status           | Proposed                                                                         |
| Roadmap             | Binary size reduction                 |
| What and why     | Reduce the size of temporary compilation artifacts so developers need less disk space |
| Zulip channel    | N/A (an existing stream can be re-used or new streams can be created on request) |
| [cargo] champion | @weihanglo
| [compiler] champion | @petrochenkov |
| [wg-compiler-performance] | @Kobzol |
| [infra] | @Kobzol |
| Funding contact | @jyn514 |

## Summary

Reduce disk space used by Rust build directories (e.g. `target/`) through iterative, measurement-guided optimizations in rustc and Cargo, focusing on intermediate artifact sizes, debuginfo, and garbage-collection.

## Motivation

Disk space usage is a recurring concern; the [2025 State of Rust survey][2025-survey] shows it as the second-most common complaint about the Rust toolchain, surpassed only by slow compilation.

[2025-survey]: https://blog.rust-lang.org/2026/03/02/2025-State-Of-Rust-Survey-results/#challenges-and-wishes-about-rust

We expect that reducing disk space usage will benefit all environments that build Rust programs, but especially:
- developers who heavily use multiple git worktrees or jj workspaces, such as in AI-first workflows or when contributing to the Rust compiler itself;
- CI jobs that cache intermediate artifacts, both in storage space and in upload/download speeds;
- persistent remote caches, such as used by Bazel and Buck2, primarily in upload/download speeds and disk IO usage.

We also theorize that by reducing disk IO, we can speed up wall time for compilation itself.
If our efforts are sufficiently fruitful, we could further decrease compilation time by allowing target directories to fit on a RAM disk (sometimes known as `tmpfs`).

Note that disk space used by build directories includes both intermediate artifacts and final artifacts.
The [binary size reduction] roadmap tracks final artifacts; this goal is focused primarily, but not solely, on intermediate artifacts.
Debuginfo size reductions will benefit both.

[Binary size reduction]: https://goals.rust-lang.org/2026/roadmap-binary-size-reduction.html

### The status quo

Compiling Rust programs takes three primary resources: time, memory, and disk space.
CPU time, and to some extent wall time and memory use, has been relentlessly optimized and micro-optimized for many years in Rustc.
Disk space usage, by contrast, has received far less attention. There have been various proposals to reduce the target directory size on the Cargo side, but little actual work due to a lack of funding, and hardly anything has been done on the Rustc side.

We believe the primary obstacles to shrinking intermediate artifacts are that it's currently difficult to measure sizes, and a lack of funding for the work.
The closest tools are `llvm-size` to show overall section sizes; `rustc -Z meta-stats` to show `.rmeta` contents; and `size:query_cache` on [perf.rust-lang.org][prlo], but none give a breakdown of `.rmeta` or incremental files by query, and are difficult to correlate across invocations.

[prlo]: https://perf.rust-lang.org

We are aware of the following ongoing work:
- The [cross workspace cache] goal and [deduplicating build artifacts across workspaces][cargo#17453], both of which reduce the number of total artifacts, but not the size of each artifact.
- The garbage-collection work to reclaim space between builds has several tracking issues ([cargo#5026], [cargo#13136], [cargo#13060]), but not a Project Goal.
- Reusing caching between `check` and `build` is being investigated in the [incremental system redesign] goal, but cannot help with initial full builds.
- [`-Zembed-metadata=no`][embed-metadata] avoids creating unnecessary metadata sections, but does not shrink the sections themselves when they are created.
- [Reduce debuginfo to `line-tables-only` in the `dev` profile][cargo#17518] reduces the size of debuginfo in the most common scenarios, but does not improve builds that use full debuginfo.

Furthermore, the Cargo team has suggested several possible improvements to disk size which could be incorporated into this goal:
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

We plan a two-phase goal.
The first is a small initiative that has only Jynn as the owner, planned at 3-6 months, with an aim of 30-40% decrease in size for fresh builds.
In the second phase, we will create a funded team of 3-5 engineers, planned at 12 months, and aim to decrease the size of the directory by 60% or more for fresh builds and by 80% or more for repeated builds across version branches.

We will work on both decreases in artifact size and garbage-collection within a target directory, such as artifacts from an earlier edit.
We will benchmark based on real crates in the Rust ecosystem, focusing on disproportionately large and widely used crates.
After benchmarking, we will work on multi-pronged approaches that allow parallel work on high-impact interventions.

At a high level, our plan is:
1. Profile disk space usage by builds, and include that data in `rustc-perf` so it is triaged alongside other performance improvements and regressions.
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

See [hackmd](https://hackmd.io/@jyn514/rkuckyShfl) for a detailed workplan.

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Measurement | @jyn514  | Phase 1 |
| Near-term improvements | @jyn514  | Phase 1 |
| Structural improvements | @jyn514  | Phase 2. Assisted by @panstromek, @weihanglo, @kobzol, and @Zeklandia. |


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

We are looking for sponsors to fund work on this goal.
We suggest funding in two phases:
first, a short-term phase, with a small group;
then a mid-term phase with a larger group.

**Phase 1: Measurements and near-term improvements**

This goal needs a minimum of (Ask) to run, which will allow us to do preliminary research and high-leverage low-hanging improvements, as described in the "Measurements" and "Near-term improvements" section of the HackMD.
Our best estimate is that this minimum funding would give around a 30% improvement in incremental target sizes over the next 6 months.

I have extensive experience working on the compiler, including the query system and integration work between Rustc and Cargo.
I also have project management experience from my time at Ferrocene,
as well as consensus building experience on policy, design, and implementation work within the Rust project.
As part of this work, I would lead implementation, mentor contributors, and coordinate between the Compiler and Cargo teams.

**Phase 2: mid-term improvements**

There are over a dozen workstreams within this proposed goal.
By myself, I cannot implement them all within a year.
Additional funding will allow me to hire and mentor other compiler and build system engineers to split up the work,
not only accelerating timelines but also completing more overall workstreams.
Furthermore, it will allow me to focus on design work rather than implementation, unlocking the larger structural improvements that require coordination between teams.

This amount of funding will allow us to reach the 60-70% (per-directory) and 80-90% (cross-directory) targets from the goal summary.

@weihanglo is a current member of the Cargo team and a member of Amazon's build performance team. He has experience implementing, reviewing, and designing changes to Cargo's garbage collection and build configuration, and has made `-Z embed-metadata` the default in Cargo, reducing metadata size by half on nightly toolchains. He has [written in detail][cargo-missing-parts] about the changes that would improve binary size.

[cargo-missing-parts]: https://weihanglo.tw/posts/2024/the-missing-parts-in-cargo/

@panstromek is a current member of the Compiler Performance working group and has ongoing work to reduce encoding sizes in rmeta files and incremental caches.
He has experience measuring, triaging, and improving the compiler's performance, and his assistance will be invaluable in optimizations within the compiler itself.

[cargo#17512]: https://github.com/rust-lang/cargo/pull/17512
[brlo]: https://blog.rust-lang.org/inside-rust/2026/08/18/reducing-target-dir-size-on-nightly/

@Kobzol has been responsible for implementing and stabilizing `-Z embed-metadata=no`, and is a current member of the Council.
`embed-metadata` has reduced the size of release builds by [almost 30%][brlo] in the past, and we are confident that further similar improvements are possible.

Zeklandia has a non-traditional programming background, but has the logistical flexibility to do short-term contract work,
and I hope to mentor her to be a long-term maintainer on these compiler subsystems.


|Purpose                                                                              |Cost                              |Funded  |Sponsor(s)                                   |
|-------------------------------------------------------------------------------------|----------------------------------|--------|---------------------------------------------|
|@jyn514 as owner (12 months, full-time)                                            |$60k in each phase, $120k total   |No      |                                             |
|@panstromek as wg-perf design contact and implementor (6 months, part-time)          |$25k in phase 2                   |No      |                                             |
|@Zeklandia as benchmark selector and analyst (variable, part-time)                   |$5k in phase 1, (Ask) in phase 2  |No      |                                             |


## Frequently asked questions

Q: Does this goal include garbage-collection across directories, such as deleting `cargo -Z script` artifacts?

No. There is [parallel work][cargo#17524] on that from the Cargo team, but it is not currently part of this goal.

[cargo#17524]: https://github.com/rust-lang/cargo/pull/17524
