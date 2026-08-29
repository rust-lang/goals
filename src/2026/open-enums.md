# Open Enums

| Metadata              |                                    |
| :--                   | :--                                |
| Contact      | @kupiakos                          |
| Status                | Accepted                           |
| Tracking issue        | [rust-lang/goals#645] |
| Other tracking issues | [rust-lang/rust#156628]            |
| Zulip channel         | N/A                                |
| [lang] champion       | @scottmcm                          |
| [compiler] champion   | @madsmtm                           |
| [opsem] champion      | @chorman0773                       |


@joshtriplett can serve as the lang champion do it if Scott can't.

## Summary

Merge and implement the [unnamed enum variants RFC][enum-rfc] to enable
ergonomic *open enums* in the language.

[enum-rfc]: https://github.com/rust-lang/rfcs/pull/3894

## Motivation

### The status quo

Rust has some mechanisms to guarantee a stable ABI for separately-compiled
types:

- `#[repr(…)]` controls the memory layout of types.

  The default is `Rust` with intentionally no stability guarantees. The `C`
  layout orders fields in memory to match declaration order.

- `extern "…" fn` controls the calling convention of a function

  The default is `"Rust"` with intentionally no stability guarantees. The `"C"`
  ABI mirrors what is the default calling convention in C for the platform.

- `#[unsafe(no_mangle)]` and `unsafe extern {…}` controls the symbols used for
  linking.

These focus on providing stability when the *same type definition* is separately
compiled with different versions of rustc. This includes the initial
[proposed][crabi] `extern "crabi" fn` and `repr(crabi)`, which provides
versioned stable layout algorithms and calling conventions.

[crabi]: https://github.com/rust-lang/rust/pull/105586

However, these do not consider stability between separately-compiled units *when
a type is extended*. While `#[non_exhaustive]` prevents breakage due to
non-exhaustive `match`es when types compile together, it does nothing to prevent
ABI breakage (and accidental UB) when a type is extended in one compiled unit
and not the other.

For a `struct`, it is simple enough prevent known future changes to a type from
affecting its layout: include an unused field to pre-allocate space of the
needed size. For an `enum`, however, there is no mechanism to pre-declare future
variants' discriminants as valid and opt-in to ABI stability.

Consider `bindgen`, which has [4 different ways] to interoperate with a C enum:

[4 different ways]: https://docs.rs/bindgen/latest/bindgen/enum.EnumVariation.html

| Variation | Description | Drawback |
| --------- | ----------- | -------- |
| Constants | Exports named integers | No strong typing or grouping - the same as `#define` |
| Module constants | The same as above but grouped in a module; `use enum_name::*` brings its values into scope | No strong typing |
| Newtype | `#[repr(transparent)] struct EnumName(Int)` | Harder to read, loses Rust enum features |
| Rust | A closed Rust `enum` with explicit `repr` | Undefined Behavior when an invalid value is trivially produced by C |

These all have their own serious drawbacks. While a newtype integer is a fine
enough way to represent an open C enum, it is no longer treated as an enum by
Rust. The choice of `enum` or `struct` reflects the author's intent and informs
how macros, tooling, and other features are supposed to interpret the type - a
set of names with associated values, or a wrapper around a plain integer. When a
`struct` is used to represent an open enumeration:

- The names aren't syntactically grouped with the type declaration.
- Code analysis and lints specific to enums are unavailable.
  - No "fill match arms" in rust-analyzer.
  - The [`non-exhaustive patterns` error][E0004] lists only integer values, and
    cannot suggest the named variants of the enum.
  - The unstable [`non_exhaustive_omitted_patterns`] lint has no easy way to
    work with this enum-alike, even though treating it like a `non_exhaustive`
    enum would be more helpful.
- It's invalid to `use` the variants like with `use EnumName::*`.
  - `bindgen` works around this by allowing the variants to be declared
    globally.
- In order for an open enum's variant name to match the normal style for an enum
  variant name, `allow(non_uppercase_globals)` is required.
- `derive`s that work with names are less useful. The built-in `derive(Debug)`
  can't know the variant names to list. Third-party macros cannot easily know
  the variant names. The `open-enum` crate, which provides an attribute macro to
  construct newtype integers from an `enum` declaration, requires a distinct
  `derive` ecosystem for operations like `TryFrom`, `Debug`, `IsKnownVariant`,
  ser/de, etc. - a worse experience than if all derives were capable of reading
  a first-class open `enum` definition.

[E0004]: https://doc.rust-lang.org/stable/error-index.html#E0004
[`non_exhaustive_omitted_patterns`]: https://doc.rust-lang.org/stable/nightly-rustc/rustc_lint_defs/builtin/static.NON_EXHAUSTIVE_OMITTED_PATTERNS.html

### What we propose to do about it

To focus on one missing piece of the ABI puzzle and improve C interop, we
propose to merge and implement the the [unnamed enum variants RFC][enum-rfc].

This allows discriminants to be declared as valid before they are represented by
a named variant in the future. By adding `_ = ..` to a field-less `repr(C)`
enum, it becomes an *open enum* and can now be safely used with C and
(ordinarily) `as` cast from `c_int`.

#### Design axioms

- **Stability**: Provide a stable ABI when an `enum` is modified and across
  compiler versions.
- **Clarity**: The valid values of an `enum` are unambiguously declared
  up-front.
- **Intent**: An `enum` captures the intent to name a set of values that may be
  exhaustively `match`ed.
- **Integration**: Integrate with the rest of the language and future
  extensions, especially lints and warnings.
- **Ergonomics**: It's nearly as easy to use an open enum as a closed enum.

#### The "shiny future" we are working towards

- Dynamically linked code is [safe and ergonomic][safe-linking] to use and
  extend.
- Users can export extensible `enum`s that may be used in dynamically-linked
  libraries, including `enum`s with fields.
- Tooling prevents accidental ABI breakage when extending any type by confirming
  a type's layout and bit validity isn't affected by a change.
- Open enums are fully-featured `enum`s with all of their tooling. Third-party
  macros recognize unnamed variants. It is just as easy to construct and
  manipulate open enums as closed enums are.
- Bindgen generates `repr(C)` open `enum`s by default, and there's rarely a
  reason to select another variation.

[safe-linking]: https://goals.rust-lang.org/2025h1/safe-linking.html

### Work items over the next year

| Task | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Integrate feedback and merge unnamed variants RFC | @kupiakos | |
| Implement and merge nightly implementation | @kupiakos | |

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Medium        | Implementation reviews                  |
| [lang]     | Medium        | Champion and (ideally) a lang meeting   |
| [libs]     | Small         | Changes to `derive`                     |
| [opsem]    | Small         | Doc changes if necessary                |
| [types]    | Small         |                                         |

## Frequently asked questions

### Why a lang feature? Isn't a macro good enough for open enums?

While a macro can be used to make declaring an integer newtype simpler, the
drawbacks of using a `struct` to represent a set of enumerated values are
extensive. It also cannot simply represent extensible enums with fields as
described in the "shiny new future".

We aim to make working with open enums ergonomic in a way they cannot be today,
even with simpler changes to the language.
