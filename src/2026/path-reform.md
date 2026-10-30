# Path reform

| Metadata         |                                           |
| :--------------- | ----------------------------------------- |
| Contact          | @clarfonthey                              |
| Status           | Proposed                                  |
| What and why     | Overhaul the standard library `Path` APIs |

## Summary

Overhaul the standard library `Path` APIs, both creating a "v2" version of the existing APIs and adding explicit platform-independent POSIX/Windows path types.

## Motivation

### The status quo

Even though the standard library `Path` and `PathBuf` types are ubiquitously used throughout the ecosystem, their current design has a number of shortcomings that make them inadequate for a large number of use cases. Issues range from clunky APIs to catastrophic performance regressions, and it's clear that a lot of work needs to be done to improve these APIs for the larger ecosystem.

One of the most relevant issues is the fact that the difference in handling between Windows and Unix paths means that, although the logic is entirely encapsulated in platform-independent Rust code, the current implementation only allows using the Windows logic on Windows targets and the Unix logic on Unix targets, meaning that tests have to be built and run for separate targets even though the code compiles just fine on both targets. Having to cross-compile just to run different parsing logic isn't great.

Additionally, due to the inherent complexity in Windows paths, APIs are forced to effectively deal with this complexity in all cases regardless of whether they care, which leads to awful performance regressions and subtle bugs. Recently, due to the advent of LLM bug scanning, a number of long-standing `Path` bugs have been revealed that further indicate these issues.

While there are a few ecosystem crates that attempt to fix these issues, the complexity of the `Path` API still represents an unreasonable hurdle into implementing wrappers over these types for various use cases, and we should attempt to offer those in the standard library itself to help mitigate a majority of cases and improve the quality of code that handles filesystem paths, which is *quite a lot of code*.

### What we propose to do about it

The most substantial, immediate work has been suggested for a very long time and actually has already been implemented by Python: we need to separate out the Windows and POSIX path types into two types that can be used and tested on all platforms. (Python calls these `WindowsPath` and `PosixPath`, although the Rust story is a bit more complicated.)

Note that these types actually should not have the same API because they will be able to explicitly take the quirks of their underlying implementations into account. For example, a large portion of the handling of the `Path::components` iterator depends on the prefix of Windows paths, and it would be nice to remove prefix handling entirely from the components iterator to remove that complex branch entirely. There could also be separate types for Windows' verbatim paths for users who explicitly want to use verbatim paths for their hygiene guarantees.

Another reasonable option is to support Unicode-only paths on both platforms, which would be useful for users who want to then convert their paths into strings and would consider non-Unicode data to simply be an error. Ecosystem crates like `camino` exist for this purpose, but this would be a standard library alternative.

One other important subtlety is the potential for paths intended to be used for `file://` URLs, since those further restrict the number of options on Windows in subtle, difficult-to-understand ways.

The options really are endless here, and part of the open-endedness here is because the clunkiness of `Path` has resulted in the ecosystem effectively *avoiding* the use of `Path` whenever possible to avoid the clunkiness, either via converting other types like `String` or `OsString` into `Path` whenever an API requires it, or potentially reimplementing path logic themselves. We would really like to help fix this conundrum and ensure that the ecosystem actually has APIs available they *want* to use, rather than those they simply *must* use.

That said, the lower bound on the work for this goal is to simply create three new `Path` APIs:

1. A "v2 `Path`" API which can potentially replace the current `Path` on a future edition, taking new designs into account.
2. A platform-independent POSIX path API which explicitly adapts the API for the handling of POSIX paths.
3. A platform-independent Windows path API which explicitly adapts the API for the handling of Windows paths.

Because the `std::path` module actually contains several types related to `Path` (like `PathBuf` and various iterators), these three APIs would actually correspond to three submodules of `std::path`, with the placeholder `std::path::v2` being able to replace the default `std::path` APIs on a future edition.

There is also a separate desire to have a distinction between relative and absolute paths at the type level, since relative paths can be represented almost platform-independently, although this is likely to be folded into the general "v2" `Path` API if it makes sense or omitted from the initial version depending on how things pan out.

### The "shiny future" we are working towards

As mentioned, there are several variants of paths that could extend into the standard library as possible extensions of the MVP of this proposal, with varying levels of desirability:

* Unicode-only flavours of path APIs
* ASCII-only path APIs
* `file://`-compatible paths (notably different on Windows)
* Verbatim-only Windows paths (implicitly backslash-only)
* Forward-slash-only Windows paths
* Custom path separators (Rust double-colon paths?)

### Work items over the next year

| Task                        | Owner(s) | Notes                       |
| --------------------------- | -------- | --------------------------- |
| `std::path::windows` ACP    | None yet |                             |
| `std::path::posix` ACP      | None yet |                             |
| `std::path::v2` ACP         | None yet |                             |
| Windows path implementation | None yet |                             |
| POSIX path implementation   | None yet |                             |

## Team asks

| Team        | Support level | Notes                              |
| ----------- | ------------- | ---------------------------------- |
| [libs]      | Large         | Review API proposals, code changes |

## Funding

| Purpose            | Cost    | Funded | Sponsor(s) |
| ------------------ | ------- | ------ | ---------- |
| Design/development | Unknown | No     |            |
