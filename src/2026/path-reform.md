# Path reform

| Metadata         |                                           |
| :--------------- | ----------------------------------------- |
| Contact          | @clarfonthey                              |
| Status           | Proposed                                  |
| What and why     | Overhaul the standard library `Path` APIs |
| [libs] champion | @clarfonthey |

## Summary

Overhaul the standard library `Path` APIs, both creating a "v2" version of the existing APIs (potentially becoming the default on a new edition) and adding explicit platform-independent POSIX/Windows path types.

## Motivation

### The status quo

Even though the standard library [`Path`] and [`PathBuf`] types are ubiquitously used throughout the ecosystem, their current design has a number of shortcomings that make them inadequate for a large number of use cases. Issues range from clunky APIs to catastrophic performance regressions, and it's clear that a lot of work needs to be done to improve these APIs for the larger ecosystem.

[`Path`]: https://doc.rust-lang.org/nightly/std/path/struct.Path.html
[`PathBuf`]: https://doc.rust-lang.org/nightly/std/path/struct.PathBuf.html

One of the most relevant issues is the fact that the difference in handling between Windows and Unix paths means that, although the logic is entirely encapsulated in platform-independent Rust code, the current implementation only allows using the Windows logic on Windows targets and the Unix logic on Unix targets, meaning that tests have to be built and run for separate targets even though the code compiles just fine on both targets. Having to cross-compile just to run different parsing logic isn't great, and this also means that, for example, a Linux-based server cannot meaningfully parse paths for a Windows-based client, and vice/versa.

Additionally, due to the inherent complexity in Windows paths and the choice to include the [`Prefix`] as a [`Component`], APIs are forced to effectively deal with this complexity in all cases regardless of whether they care, which leads to awful performance regressions and subtle bugs. Recently, due to the advent of LLM bug scanning, a number of long-standing [`Path`] bugs have been revealed that further indicate these issues. [(Examples in the appendix.)](#appendix-correctness-bugs)

[`Prefix`]: https://doc.rust-lang.org/nightly/std/path/enum.Prefix.html
[`Component`]: https://doc.rust-lang.org/nightly/std/path/enum.Component.html

While there are a few ecosystem crates that attempt to fix these issues, the complexity of the `Path` API still represents an unreasonable hurdle into implementing wrappers over these types for various use cases, and we should attempt to offer those in the standard library itself. This will dramatically increase the quality of code that handles filesystem paths, which is *quite a lot of code*, but unfortunately not very easy to automatically test; most code cares more about actual file I/O than the paths themselves, which they take for granted.

### What we propose to do about it

The most substantial, immediate work has been suggested for a very long time and actually has already been implemented by Python: we need to separate out the Windows and POSIX path types into two types that can be used and tested on all platforms. (Python calls these `WindowsPath` and `PosixPath`, although the Rust story is a bit more complicated.)

Note that these types actually should not have the same API because they will be able to explicitly take the quirks of their underlying implementations into account. For example, a large portion of the handling of the `Path::components` iterator depends on the prefix of Windows paths, and it would be nice to remove prefix handling entirely from the components iterator to remove that complex branch entirely. There could also be separate types for Windows' verbatim paths for users who explicitly want to use verbatim paths for their hygiene guarantees.

The options really are endless here, and part of the open-endedness here is because the clunkiness of `Path` has resulted in the ecosystem effectively *avoiding* the use of `Path` whenever possible to avoid the clunkiness, either via converting other types like `String` or `OsString` into `Path` whenever an API requires it, or potentially reimplementing path logic themselves. We would really like to help fix this conundrum and ensure that the ecosystem actually has APIs available they *want* to use, rather than those they simply *must* use.

That said, the lower bound on the work for this goal is to simply create three new `Path` APIs:

1. A "v2 `Path`" API which can potentially replace the current `Path` on a future edition, taking new designs into account.
2. A platform-independent POSIX path API which explicitly adapts the API for the handling of POSIX paths.
3. A platform-independent Windows path API which explicitly adapts the API for the handling of Windows paths.

Because the `std::path` module actually contains several types related to `Path` (like `PathBuf` and various iterators), these three APIs would actually correspond to three submodules of `std::path`, with the placeholder `std::path::v2` being able to replace the default `std::path` APIs on a future edition.

There is also a separate desire to have a distinction between relative and absolute paths at the type level, since relative paths can be represented almost platform-independently, although this is likely to be folded into the general "v2" `Path` API if it makes sense or omitted from the initial version depending on how things pan out.

#### Implementation details

Although it's not mentioned here, part of the likely design will involve further separating these three APIs into *relative* and *absolute* paths, since relative path handling is effectively the same across platforms except for the choice of path separator as a forward or back slash. This would also allow fixing APIs like `PathBuf::push` to make more sense: instead of allowing pushing *any* path, you just can push relative paths, with an absolute path always simply replacing the existing path.

Additionally, the "components" of a path will only be retrievable from *relative* paths, meaning that absolute paths must first be split into an `Option<Prefix>` and a relative path before iterating. This will dramatically simplify prefix-parsing logic across platforms, and ensure that iterators remain fast instead of having to work around bidirectional iteration with loads of special cases. Additionally, it's likely that the "empty path" will simply be disallowed, to ensure that there doesn't need to be any extra logic to handle whether a path starts with a current-directory component or not.

### The "shiny future" we are working towards

Ideally, the future allows us to expand the existing `Path` APIs in the standard library to offer even more types, or at least make them easier for ecosystem crates to support. Below are a few examples of potential new APIs to explore, either in standard library crates or downstream:

#### Unicode paths, and restricting valid paths

This is useful for users who want to then convert their paths into strings and would consider non-Unicode data to simply be an error, which is a very common use case. (Many projects encode data in file names, and when these names are `String`s, they should be Unicode.)

Ecosystem crates like [`camino`] exist for this purpose, but it could be useful to have a standard library alternative. Note that there is precedent for this already in the standard library, with functions like `std::env::args` (forcing Unicode for command-line arguments) and `std::env::vars` (forcing Unicode for environment variables).

Additionally, a general mechanism to exclude certain characters from paths would be useful: for example, Windows disallows many characters in paths by default, but this is also present on targets like Android, and can potentially depend on the file system being used (which means it can't simply be inferred by the target).

[`camino`]: https://docs.rs/camino/latest/camino/

#### `file://`-compatible paths and URLs

While URLs are probably out of scope for the standard library, the `file://` scheme makes sense enough that we would probably want to account for paths that are valid to use in these types of URLs, even if we omit the `file://` scheme prefix.

Note that this is particularly difficult on Windows, since it's incredibly unclear what is allowed on Windows, and many specs lead this sort of information out. For example, while verbatim paths on Windows explicitly require backslashes, [`file://` explicitly requires replacing them with forward slashes](https://datatracker.ietf.org/doc/html/rfc8089#appendix-E.3.2), which is strictly incompatible with normal Windows paths.

#### Normalized paths

Right now, one massive issue with `Path` is the fact that equality and ordering are explicitly very expensive due to the normalisation of components. Consider the following cases from the standard library on which paths are considered the same:

| `lhs`        | `rhs`        | `Path::new(lhs) == Path::new(rhs)` |
| ------------ | ------------ | ---------------------------------- |
| `./x`        | `x`          | `true`                             |
| `x/.`        | `x`          | `true`                             |
| `x/`         | `x`          | `true`                             |
| `C:/`        | `C:`         | `false`                            |
| `C:/`        | `C:\`        | `true`                             |
| `\\?\C:\`    | `C:\`        | `false`                            |
| `a/../b`     | `b`          | `false`                            |
| `C:\Windows` | `c:\windows` | `false`                            |
| `C:\Windows` | `c:/Windows` | `true`                             |

All of these questions are hard to answer without an explicit API that guarantees path normalisation. And so, it would be nice to have a "normalized path" type where an invariant is upheld that two paths are equal if their strings are identical, and this would require a number of knobs to configure like case-sensitivity, Unicode normalisation, etc. While things like case folding are probably out of scope here, the ability to ensure that e.g. there are no current-directory or non-leading previous-directory components present could be quite useful.

Additionally, for the sake of safety when extracting archives, one necessary but not sufficient property for safety is excluding previous-directory components.

#### Custom path separators

Really, the split-at-separator logic could be useful for a broad number of other cases, like being able to efficiently parse and compare Rust paths. Being able to replace slashes with double-colons, for example, could be nice, and we should try and see what kinds of ways we can implement this efficiently and let the ecosystem use that code.

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

Minimum funding would enable @clarfonthey and potentially others to devote time to actually design and implement these new APIs. While it may be possible also to fund other library team members to review the proposals, that is probably better funded via a mechanism such as the [Rust Foundation Maintainers Fund](https://rust-lang.org/funding/#rfmf).

| Purpose            | Cost    | Funded | Sponsor(s) |
| ------------------ | ------- | ------ | ---------- |
| Design/development | Unknown | No     |            |

## Appendix: correctness bugs

As part of preparing this goal, an [`A-path`] label was added to the `rust-lang/rust` repo to track `std::path` issues separate from `std::io` and `std::fs` issues. Additionally, these are now auto-labelled on PRs based upon the modified modules. This should hopefully help longer-term to track issues with `Path` and its related types more easily.

[`A-path`]: https://github.com/rust-lang/rust/issues?q=state%3Aopen%20label%3AA-path

This list is taken from [the intersection of `A-path` and `C-bug` labels][correctness-bugs]:

[correctness-bugs]: https://github.com/rust-lang/rust/issues?q=label%3AA-path+label%3AC-bug

* 2016: [#31789: Windows paths should not be prefixed with `\\?\` when displayed](https://github.com/rust-lang/rust/issues/31789)
* 2016: [#32497: `PathBuf::set_extension` accepts an extension containing slashes](https://github.com/rust-lang/rust/issues/32497)
* 2016: [#36861: `parent()` returns `Some("")` for single-component relative paths](https://github.com/rust-lang/rust/issues/36861)
* 2022: [#93586: `Path::strip_prefix` may panic when a path consists only of a prefix component](https://github.com/rust-lang/rust/issues/93586)
* 2024: [#126291: `as_mut_vec_for_path_buf` in windows breaks UTF-8 `is_known_utf8` assumption](https://github.com/rust-lang/rust/issues/126291)
* 2024: [#132141: `Path::is_absolute` is wrong on hermit](https://github.com/rust-lang/rust/issues/132141)
* 2025: [#136617: `Path`/`OsStr` incorrectly handle `Formatter` flags](https://github.com/rust-lang/rust/issues/136617)
* 2025: [#146183: Checking a `&PathBuf` and a `&str` for equality leads to a stack overflow](https://github.com/rust-lang/rust/issues/146183)
* 2025: [#148267: `Path::strip_prefix()` improperly strips trailing slash](https://github.com/rust-lang/rust/issues/148267)
* 2026: [#152804: `std::path::Display` regressed in 1.95.0 Nightly](https://github.com/rust-lang/rust/issues/152804)
* 2026: [#154164: `std::Path` parses UNC with forward slashes incorrectly on x86_64-pc-cygwin](https://github.com/rust-lang/rust/issues/154164)
* 2026: [#154521: `Path` comparison is obscenely slow](https://github.com/rust-lang/rust/issues/154521)
* 2026: [#161651: `Eq` returns `true` but hashes are different for some verbatim paths on Windows](https://github.com/rust-lang/rust/issues/161651)
* 2026: [#164467: `Path::components().collect::<PathBuf>()` does not preserve components of verbatim paths](https://github.com/rust-lang/rust/issues/164467)
* 2026: [#164472: `std::path::Components` have disagreeing impls of `PartialEq` vs `PartialOrd`/`Ord` on windows](https://github.com/rust-lang/rust/issues/164472)
