# Highlights from the 2026 goals

There are a total of (((#GOALS))) planned for this year. That's a lot! You can see the complete list, but to help you get a handle on it, we've selected a few to highlight. These are goals that will be stabilizing this year or which we think people will be particularly excited to learn about.

**Important:** You have to understand the nature of a Rust goal. Rust is an open-source project, which means that progress only happens when contributors come and *make* it happen. When the Rust project declares a goal, that means that (a) contributors, who we call the *task owners*, have said they want to do the work and (b) members of the Rust team members have promised to support them. Sometimes those task owners are volunteers, sometimes they are paid by a company, and sometimes they supported by grants. But no matter which category they are, if they ultimately are not able to do the work (say, because something else comes up that is higher priority for them in their lives), then the goal won't happen. That's ok, there's always next year!

## Running Rust scripts will get more convenient with *cargo script*

(((HIGHLIGHT TABLE: Cargo script)))

*People involved: (((HIGHLIGHT CREDITS: Cargo script)))*

---

"Cargo script" let's you create a single file that specifies both a Rust program and the dependencies it needs and then execute that program with one convenient command. For example, you can now take a Rust file like this:

```rust
#!/usr/bin/env cargo
---
edition: 2024
[dependencies]
reqwest = { version = "0.12", features = ["blocking"] }
---

fn main() {
    let body = reqwest::blocking::get("https://httpbin.org/ip")
        .unwrap()
        .text()
        .unwrap();
    println!("My IP info: {body}");
}
```

and run it with `cargo my_ip.rs`.  Or, thanks to the `#!` line, you can just run `./my_ip.rs`.

This feature makes good use of [one of the things we found when doing our research for the Vision Doc](https://blog.rust-lang.org/2025/12/19/what-do-people-love-about-rust/#but-what-they-love-is-the-sense-of-empowerment-and-versatility): that people love Rust not only because it helps them build foundational software, but because it's a expressive and productive enough that "you can write everything from the top to the bottom of your stack in it" (-- Rust expert and consultant focused on embedded and real-time systems). Until now, the fly in the ointment was that packaging up a Rust package required several files and required people to do a separate compilation step. Cargo script solves that problem.

## The borrow checker will be more flexible with *Polonius alpha*

(((HIGHLIGHT TABLE: Polonius)))

*People involved: (((HIGHLIGHT CREDITS: Polonius)))*

---

The "Polonius Alpha" work represents the final completion of the original promise from the [2018 Non-lexical Lifetimes RFC](https://rust-lang.github.io/rfcs/2094-nll.html). That RFC originally planned to address three problematic patterns -- but ultimately, for efficiency reasons, we were only able to fix two. In the meantime, for the last several years, we have been pursuing work on Polonius, a next generation borrow checker formulation, that aims to close this gap and more.

The Polonius Alpha rules extend the borrow checker to accept the so-called ["Problem Case #3"](https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions) that NLL ultimately failed to solve. This case occurs when you have conditional control flow across functions. For example, in this case the call to `map.get_mut(&key)`, the borrow of `map` is only "live" in the `Some` branch, where it is returned (and hence must outlive `'r`). But because of imprecision in the borrow checker, the borrow winds up being enforced in the `None` branch as well, resulting in an error:

```rust
fn get_default<'r,K:Hash+Eq+Copy,V:Default>(
    map: &'r mut HashMap<K,V>,
    key: K,
) -> &'r mut V {
    match map.get_mut(&key) { // ──────────────────┐ 'r only needs to
        Some(value) => value,              // ◄────┘ be valid here...
        None => {                          //      │
            map.insert(key, V::default()); //      │
            //  ^~~~~~ ERROR               //      │
            map.get_mut(&key).unwrap()     //      │
        }                                  //      │
    }                                      //      │ ...but today it covers
}                                          // ◄────┘ all this
```

Under Polonius Alpha, this code compiles.

Polonius Alpha is part of a larger roadmap called [the Borrow-Checker Within](./roadmap-borrow-checker-within.md) that we expect to be driving over the next few years. This year, another part of that work is including Polonius Alpha in [a-mir-formality](https://github.com/rust-lang/a-mir-formality/), the [types team's](https://rust-lang.org/governance/teams/compiler/#team-types) (in-progress) specification for how the Rust type system works. As part of another goal, we are planning to [integrate a-mir-formality into the Rust reference](./a-mir-formality.md). This would make Polonius the first version of the borrow checker whose behavior is specified outside of the Rust compiler.

## Change const evaluation to support *traits*, and *reflection*, allow *structs/enums* as const parameter types

(((HIGHLIGHT TABLE: Const and reflection)))

*People involved: (((HIGHLIGHT CREDITS: Const and reflection)))*

---

This year we'll be extending Rust's support for const evaluation in several ways. To start, you'll be able to use structs and enums as the values for const generics, not only integers. So where today you can write `Array<3>`, you'll be able to write something like this:

```rust
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub fn process<const D: Dimensions>(data: &[f32]) {
    // ...
}

fn main() {
    process::<{ Dimensions { width: 1920, height: 1080 } }>(&data);
}
```

You'll also be able to use associated constants as const generic arguments, like `Buffer<T::MAX_SIZE>`.

Next, we are integrating `const` into the trait system. When you implement a trait, you'll be able to provide a `const` impl which means that the methods in the trait are all const-compatible. `const fn` can then use bounds like `T: const Display` to indicate that they need a type with a const-compatible impl or `T: [const] Display` to indicate that they need a const-compatible impl when called in a const context. Const traits are particularly helpful because they allow you to use builtin language constructs like `?` and `for` loops:

```rust
const fn sum_up<I: [const] Iterator<Item = i32>>(iter: I) -> i32 {
    let mut total = 0;
    for val in iter {
        total += val;
    }
    total
}
```

Finally, we're beginning early experimental work on compile-time reflection — the ability for const functions to inspect type information. It's too early to promise specifics, but the long-term vision is things like serialization working without derive macros.

## Ergonomic ref-counting and (maybe) async traits

(((HIGHLIGHT TABLE: Async and ergonomic RC)))

*People involved: (((HIGHLIGHT CREDITS: Async and ergonomic RC)))*

---

We have a lot of ongoing plans to improve the async Rust experience, but the two most likely to hit stable are [more ergonomic ref-counting](./ergonomic-rc.md) and [extensions to async fn in traits](./afidt-box.md).

The [ergonomic ref-counting](./ergonomic-rc.md) discussion has gone through [many stages](https://smallcultfollowing.com/babysteps/series/ergonomic-rc/), but one solid step everyone agrees on is making it (a) more obvious when you are sharing two handles to the same object vs doing a deep clone, via the `Share` trait, and (b) more ergonomic to capture clones into closures and async blocks with `move($expr)` expressions:

```rust
// Today: awkward temporary variables
let tx_clone = tx.clone(); // am I deep cloning or what?
tokio::spawn(async move {
    send_data(tx_clone).await;
});

// With Share + move expressions: inline and clear
tokio::spawn(async {
    send_data(move(tx.share())).await;
}); //        ---------------- capture a shared handle
```

We also plan to cut a "practical path" to support [invoking async fns through `dyn Trait`](./afidt-box.md). The initial version would be limited to boxed futures but the goal is to be forwards-compatible with the ongoing [in-place initialization](./in-place-init.md) designs for non-boxed allocation (e.g., stack). The RFC for this hasn't been written yet, and the proposal includes some new syntax, so that could be spicy! Stay tuned.

## Try, never, extern types, oh my!

(((HIGHLIGHT TABLE: Try, never, extern types)))

*People involved: (((HIGHLIGHT CREDITS: Try, never, extern types)))*

---

Three long-awaited features are making their way toward stabilization this year.

The [`Try` trait](./stabilize-try.md) customizes the behavior of the `?` operator, letting you use it with your own types beyond `Result` and `Option`. For example, you could define a `TracedResult` that automatically captures the source location each time an error is bubbled up with `?`:

```rust
fn read_list(path: PathBuf) -> TracedResult<Vec<i32>> {
    let file = File::open(path)?;  // captures location
    Ok(read_number_list(file)?)    // captures location
}
```

No more choosing between readable error handling and useful diagnostics.

The [never type `!`](./stabilize-never-type.md) has been unstable for *ten years*. It represents computations that never produce a value — like functions that always panic or loop forever. The final blockers are being resolved, and stabilization is in sight.

Finally, the [Sized trait hierarchy](./scalable-vectors.md) work will stabilize a richer set of sizing traits, which unblocks [extern types](https://github.com/rust-lang/rfcs/pull/1861) — another long-requested feature. Today, `?Sized` conflates "unsized but has metadata" with "truly sizeless." The new hierarchy distinguishes these cases. This same work is also laying the foundation for scalable vector support (Arm SVE), where vector sizes depend on the CPU rather than being fixed at compile time.

## Going "beyond the `&`" with better integration for custom pointer types

(((HIGHLIGHT TABLE: Custom pointer types)))

*People involved: (((HIGHLIGHT CREDITS: Custom pointer types)))*

---

Three goals this year are working to make it possible for user-defined types to be used in all the ways that you can use `Box`, `Arc`, and `&`.

[Arbitrary self types](./arbitrary-self-types.md) lets you use custom smart pointers as method receivers. With the `Receiver` trait and `derive(CoercePointee)`, your pointer types work just like `Box` or `Arc` — including method dispatch and coercion to `dyn Trait`:

```rust
impl Person {
    fn biometrics(self: &SmartPointer<Self>) -> &Biometrics {
        ...
    }
}

let person: SmartPointer<Person> = get_data();
let bio = person.biometrics(); // just works
```

[Reborrow traits](https://goals.rust-lang.org/2026/reborrow-traits.html) allow custom pointers to be reborrowed, just like mutable references. When working with Pin, for example, you should no longer have to call `pinned_ref.as_mut()` to fix lifetime issues.

We are also continuing our experimental work to support [custom field projections](./field-projections.md) — accessing fields *through* a smart pointer. Today, `&x.field` gives you `&Field`, but there's no equivalent for `NonNull`, `Pin`, or custom pointer types. The field projections design is exploring a "virtual places" approach that would make this work generically. The goal for this year is a compiler experiment on nightly and draft RFCs, with the [beyond-refs wiki](https://rust-lang.github.io/beyond-refs/) documenting the design space.

These goals spun out from the ongoing work to support the needs of the [Rust for Linux](./roadmap-rust-for-linux.md) project and are part of the [Beyond the `&`](./roadmap-beyond-the-ampersand.md) roadmap.

## Build it your way with build-std

(((HIGHLIGHT TABLE: Build-std)))

*People involved: (((HIGHLIGHT CREDITS: Build-std)))*

---

A new version of [build-std](./build-std.md) is expected to hit nightly this year. Build-std lets Cargo rebuild the standard library from source, which unlocks things like using std with tier three targets, rebuilding with different codegen flags, and stabilizing ABI-modifying compiler flags. It's particularly valuable for embedded developers, where optimizing for size matters and targets often don't ship with a pre-compiled std.

An unstable `-Zbuild-std` flag has existed for a while, but this new design — progressing through a series of RFCs ([one accepted](https://github.com/rust-lang/rfcs/pull/3873), [two more](https://github.com/rust-lang/rfcs/pull/3874) [in review](https://github.com/rust-lang/rfcs/pull/3875)) — has a path to stabilization. Build-std is also part of the [Rust for Linux](./roadmap-rust-for-linux.md) roadmap.

## Closing soundness bugs and supporting new lang features with a new trait solver

(((HIGHLIGHT TABLE: Next-generation trait solver)))

*People involved: (((HIGHLIGHT CREDITS: Next-generation trait solver)))*

---

This year, the Rust types team plans to stabilize the [next-generation trait solver](./next-solver.md). This solver is a ground-up rewrite of the core engine that decides whether types satisfy trait bounds, normalizes associated types, and more. The types team has been working on it since late 2022, and it already powers coherence checking as of Rust 1.84. The goal for this year is to stabilize it for use across all of Rust and remove the old implementation.

This goal may not *sound* like it's going to impact your life, but finishing the new solver unblocks a *lot* of stuff. To start, it allows us to make progress on the [Project Zero](./roadmap-project-zero.md) roadmap, which aims to fix every known type system soundness bug. It also unblocks long-desired features like implied bounds, cyclic trait matching, and features needed by the [Just add async](./roadmap-just-add-async.md) roadmap.
