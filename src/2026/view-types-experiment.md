# View types experiment

| Metadata            |                              |
| :------------------ | :--------------------------- |
| Contact             | @scrabsha                    |
| Status              | Accepted                     |
| Tracking issue      | [rust-lang/goals#702]        |
| Zulip channel       | N/A                          |
| [compiler] champion | @oli-obk                     |
| Funding contact     | [Hexcat](https://hexcat.nl/) |
| Roadmap             | The Borrow Checker Within    |

## Summary

Add view types support to `rustc` as well as a few syntaxes proposed by project
members for experimentation purposes.

## Motivation

Rust currently does not allow tracking borrows across function calls: when a
reference to an object is passed to a function, the borrow checker considers the
entire object to be used by the function. Developers work around this by passing
individual fields of a struct as arguments instead of passing the whole struct.

View types solve this by adding type-level annotations that restrict which
fields of a struct can be accessed in a function body. The borrow checker allows
other fields to be borrowed at the same time on the caller's side.

The ideal syntax for view types is still debated. We aim to implement a few of
the competing syntaxes so that people can experiment with them and form
meaningful opinions.

### The status quo

As stated in the previous section, the borrow checker cannot reason about
individual fields of a struct being used or not by a specific function. It ends
up rejecting code that would respect borrow-checking invariants, had the
function call been inlined. Let's consider the following code as an example:

```rust
struct Foo {
    a: usize,
    b: u32,
}

let mut foo = Foo { a: 101, b: 42 };

let a = &mut foo.a;
let b = &mut foo.b; // (1)
*b += 1;            // (2)
*a += 1;
```

###### ([Playground link](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=3cc9293d8694f8636ff7e3384ae9927c))

This code compiles, let's try extracting the statements (1) and (2) to their own
function:

```rust
struct Foo {
    a: usize,
    b: u32,
}

fn increment_b(foo: &mut Foo) {
    let b = &mut foo.b;
    *b += 1;
}

let mut foo = Foo { a: 101, b: 42 };

let a = &mut foo.a;
increment_b(&mut foo);
*a += 1;
```

###### ([Playground link](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=43c70f64783779f2b34c9fedf994627e))

This fails to compile. `rustc` version 1.96.0 emits the following error message:

```text
error[E0499]: cannot borrow `foo` as mutable more than once at a time
  --> src/main.rs:15:13
   |
14 | let a = &mut foo.a;
   |         ---------- first mutable borrow occurs here
15 | increment_b(&mut foo);
   |             ^^^^^^^^ second mutable borrow occurs here
16 | *a += 1;
   | ------- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```

People generally fix this by making sure `increment_b` does not borrow "too much
data" and pass only the fields of `Foo` that the function really needs:

```rust
struct Foo {
    a: usize,
    b: u32,
}

fn increment_b(b: &mut u32) {
    *b += 1;
}

let mut foo = Foo { a: 101, b: 42 };

let a = &mut foo.a;
increment_b(&mut foo.b);
*a += 1;
```

###### ([Playground link](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=5a1ba89055d02dd723f5b98fd5d6f609))

This has the following drawbacks:

- This creates unnecessary friction during refactorings
- This requires adding one argument for each field that is used (instead of a
  single argument for the whole struct)
- IDEs do not support this, they generate code that does not compile
- Methods have to be turned into associated functions or free functions, which
  sometimes makes the code harder to understand

### What we propose to do about it

#### General solution

We introduce view types: a zero-cost wrapper over an actual type that restricts
the set of fields that can be accessed.

In the previous example, view types would make the code clearer:

```rust
struct Foo {
    a: usize,
    b: u32,
}

fn increment_b(foo: &mut Foo.{ b }) {
    let b = &mut foo.b;
    *b += 1;
}

let mut foo = Foo { a: 101, b: 42 };

let a = &mut foo.a;
increment_b(&mut foo);
*a += 1;
```

Three things happen here:

- `increment_b` takes an `&mut Foo.{ b }`: a mutable reference to a value of
  type `Foo` of which only the field `b` can be accessed
- The `&mut foo` that is passed to `increment_b` is coerced to `&mut Foo.{ b }`
- The borrow checker accepts this code because it now understands that
  `&mut foo as &mut Foo.{ b }` only requires mutably borrowing `foo.b`.

##### The `Contiguous` marker trait

Passing a view type as argument to `memcpy` is unsound:

```rust
fn copy<T>(src: T, dst: &mut T) {
    unsafe extern "C" {
        fn memcpy(n: usize, dest: *mut c_void, src: *const c_void);
    }

    unsafe {
        memcpy(
            size_of::<T>(),
            &raw mut *dst as *mut c_void,
            &raw const src as *const c_void,
        );
    }
}
```

The `Contiguous` marker trait is implemented for any type that is not a view
type. An implicit trait bound as also added (much like `Sized` currently). This
way, calling `copy::<Foo.{}>` would trigger a compilation error.

#### Limitations

This Project Goal aims to allow for experimentation around view types, not to
allow people to use view types in production. As such, the following artificial
limitations are introduced:

- Publicly exposing a view type in a crate's API is a hard error, even if the
  fields mentioned are `pub`.
- Interactions with pattern types are not considered.

#### Syntax

To facilitate testing, the following syntaxes will be added:

- `Ty.{ <fields> }` syntax (from
  [_Maximally minimal view types, a follow-up_](https://smallcultfollowing.com/babysteps/blog/2026/03/22/max-min-view-types-followup/))
- `Ty.{ .. }` as a shorthand for `Ty` (from
  [_Maximally minimal view types_](https://smallcultfollowing.com/babysteps/blog/2026/03/21/view-types-max-min/))
- `Ty.field` as a shorthand for `Ty.{ field }` (suggested by
  [Benno Lossin](https://github.com/BennoLossin) at Rust All Hands 2026)
- Disallowing `Ty.{ <fields> }` in function prototypes, requiring them to be
  introduced via type aliases (akin to view groups) (suggested by
  [Benno Lossin](https://github.com/BennoLossin) at Rust All Hands 2026)
- `&mut { <fields> } Ty` (from
  [_Syntactic Musings on View Types_](https://blog.yoshuawuyts.com/syntactic-musings-on-view-types/))

### Impact

Encapsulation is notoriously hard to achieve in Rust compared to other
languages. Programmers split big functions into multiple smaller functions, each
with a clear purpose, and suddenly the code stops compiling, because the
compiler cannot track complex borrows _across_ functions, while it could track
them _within_ a function. Code that feels clean and natural to a programmer gets
rejected by the compiler.

View types address this by adding a syntax that allows programmers to express
borrows of struct fields across functions. This means that more code that
_feels_ straightforward and natural to write, actually compiles, making the
language easier to learn and adopt.

### Work items over the next year

| Task                                         | Owner(s)  | Notes |
| -------------------------------------------- | --------- | ----- |
| First implementation of view types           | @scrabsha |       |
| Implementation of the alternative syntaxes   | @scrabsha |       |
| `Contiguous` marker trait and implicit bound | @scrabsha |       |

## Team asks

| Team       | Support level | Notes                                               |
| ---------- | ------------- | --------------------------------------------------- |
| [compiler] | Medium        | Code review, questions                              |
| [lang]     | Small         | Suggestion of alternative syntaxes                  |
| [types]    | Small         | Add the `Contiguous` marker trait and implied bound |

## Funding

| Purpose        | Cost | Funded | Sponsor(s) |
| -------------- | ---- | ------ | ---------- |
| Implementation | Ask  | No     |            |

## Frequently asked questions

(None?)
