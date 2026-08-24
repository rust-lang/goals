# Ergonomic ref-counting

| Metadata         |                                               |
|:-----------------|-----------------------------------------------|
| Point of contact | @nikomatsakis                                 |
| Status           | Accepted                                      |
| What and why     | A `Share` trait for clone-as-alias types and `move($expr)` for precise closure capture control |
| Roadmap          | [Just add async](./roadmap-just-add-async.md) |
| Tracking issue   | [rust-lang/goals#107]            |
| Highlight        | Async and ergonomic RC                         |
| Zulip channel    | N/A                                           |
| [lang] champion  | @nikomatsakis                                 |

## Summary

Implement and prototype two foundational improvements for ergonomic ref-counting: (1) a `Share` trait that semantically identifies types where cloning creates an alias to the same underlying value, and (2) move expressions (`move($expr)`) that allow precise control over what closures capture and when. These changes lay groundwork for future ergonomic improvements while delivering immediate value, with prototypes targeted for summer 2026.

## Motivation

### The status quo

Working with ref-counted data in Rust is a well-documented pain point. The problem affects high-level GUI applications (Dioxus, Sycamore), async network services (tokio), language interop (PyO3), and even the Rust compiler itself. Projects have gone to great lengths to work around it, from arena-based designs to custom preprocessors.

This goal represents the culmination of extensive design exploration. After [RFC #3680] proposed `.use` syntax and received mixed feedback, we spent 2025H2 deeply exploring the design space through [a series of blog posts][blog-series] and community discussions. Those discussions led to reframing the goal, focusing first on solutions **"low-level enough for a kernel, usable enough for a GUI"**.

[blog-series]: https://smallcultfollowing.com/babysteps/series/ergonomic-rc/

### What we propose to do about it

We are proposing to move forward with two features

1. a `Share` trait that semantically identifies types where cloning creates an alias to the same underlying value;
2. move expressions (`move($expr)`) that allow precise control over what closures capture and when.

These two features make working with ref-counted data more ergonomic while still retaining Rust's traditional guarantee that all ref counts are visible. Merging these two features is not meant to imply that we will not make other improvements targeting ref-counted data in the future, rather the idea is that these features are a solid step forward, so we should take that, and then assess what else may be needed.

### Design axioms

* **Low-level enough for a kernel, usable enough for a GUI.** Solutions should make low-level details visible for those who need them while being ergonomic enough for high-level applications.

* **Semantic over operational.** Traits should be defined by what they *mean*, not just what they *cost*. `Share` means "creates an alias," not "is cheap to clone."

#### The `Share` trait

A new trait that identifies types where cloning creates an alias to the same underlying value:

```rust
trait Share: Clone {
    fn share(&self) -> Self {
        self.clone()
    }
}

impl<T: ?Sized> Share for Arc<T> {}
impl<T: ?Sized> Share for Rc<T> {}
impl<T: ?Sized> Share for &T {}
impl<T> Share for mpsc::Sender<T> {}
```

The `share()` method is semantically equivalent to `clone()` but signals to readers that this creates an alias, not an independent copy. Types like `Arc`, `Rc`, channel senders, and shared references would implement `Share`. Types like `Vec` or `String` would not—cloning those creates independent values.

This addresses a real confusion today: when you see `map.clone()`, you can't tell if you're creating a second handle to the same map or a deep copy. With `Share`, you'd write `map.share()` for the former, making the code's intent clear.

#### Move expressions

Within closures and async blocks, `move($expr)` evaluates the expression at closure creation time and captures the result:

```rust
// Today: awkward temporary variables
let tx_clone = tx.clone();
tokio::spawn(async move {
    send_data(tx_clone).await;
});

// With move expressions: inline and clear
tokio::spawn(async {
    send_data(move(tx.clone())).await;
});
```

This generalizes Rust's existing closure model. Rather than having separate "ref closures" and "move closures," you have closures where some captures use `move()`. The `move ||` syntax becomes shorthand for "use `move()` everywhere."

Combined with `Share`:

```rust
tokio::spawn(async {
    do_something(move(self.some_a.share()), move(self.some_b.share()));
});
```

### Work items over the next year

#### Add a `Share` trait

A trait that identifies types where cloning creates an alias to the same underlying value, like `Arc`, `Rc`, and shared references.

| Task                                            | Owner(s)      | Notes                          |
| ----------------------------------------------- | ------------- | ------------------------------ |
| RFC for `Share` trait                           | @nikomatsakis | Define semantics, stdlib impls |
| Implement `Share` trait                         | @spastorino, @TaKO8Ki |                         |
| Prepare stabilization PR for `Share` trait      | @spastorino, @TaKO8Ki |                         |

#### Support `move(...)` expressions in closures

Precise control over what closures capture and when, eliminating the need for awkward clone-into-temporary patterns.

| Task                                            | Owner(s)      | Notes                          |
| ----------------------------------------------- | ------------- | ------------------------------ |
| RFC for move expressions                        | @nikomatsakis | Closure desugaring semantics   |
| Implement move expressions                      | @spastorino, @TaKO8Ki |                         |
| Prepare reference changes                       | @nikomatsakis |                                |
| Prepare stabilization PR for `move` expressions | @spastorino, @TaKO8Ki |                         |

**Target:** Working prototypes on nightly by summer 2026.

## Team asks

| Team        | Support level | Notes   |
| ----------- | ------------- | ------- |
| [compiler]  | Small         | Reviews |
| [lang]      | Medium        |         |
| [lang-docs] | Small         |         |
| [libs]  | Small         | Reviews of RFC and API surface area |

## Frequently asked questions

### This goal looks very different than I remembered. What has changed?

The goal has taken a long journey:

**2024H2:** Jonathan Kelley from Dioxus wrote a [blog post about high-level Rust][dioxus-post] that sparked the ergonomic ref-counting effort as a project goal.

**2025H1:** [RFC #3680] proposed `.use` syntax and `use ||` closures. Santiago Pastorino implemented experimental support on nightly. Community feedback was positive about addressing the problem but raised concerns about whether adding more required syntax actually improves ergonomics.

**2025H2:** Through design meetings, the RustConf Unconf, and extensive blogging, we explored the design space more deeply. Key realizations emerged:

* **Semantic over operational:** Rather than defining a trait by "what is cheap to clone" (operational), we should focus on "what does cloning mean" (semantic). When you clone an `Arc`, you get a second *handle to the same value*—this "entanglement" is the key property.

* **Explicit can be ergonomic:** After conversations with Josh Triplett, we concluded that some applications genuinely need to track where aliases are created. The goal should be making explicit code ergonomic, not making everything implicit.

* **Move expressions generalize nicely:** The `move($expr)` syntax elegantly extends Rust's existing closure model rather than adding a parallel system.

[dioxus-post]: https://dioxus.notion.site/Dioxus-Labs-High-level-Rust-5fe1f1c9c8334815ad488410d948f05e

### Can you give me an example of how these features will help?

The "Cloudflare example" from the Dioxus blog post illustrates the current pain:

```rust
let _some_a = self.some_a.clone();
let _some_b = self.some_b.clone();
let _some_c = self.some_c.clone();
tokio::task::spawn(async move {
    do_something(_some_a, _some_b, _some_c)
});
```

With `Share` and move expressions:

```rust
tokio::task::spawn(async {
    do_something(
        move(self.some_a.share()),
        move(self.some_b.share()),
        move(self.some_c.share()),
    )
});
```

This is more concise, keeps the cloning visible, and eliminates the awkward temporary variables.

### Why `Share` instead of `Alias` or `Handle`?

We considered several names. `Handle` is a good noun but awkward as a verb ("handle this value?"). `Alias` works as both noun and verb but feels slightly technical. `Share` is common, intuitive, and already used in Rust (`&T` is a "shared reference"). The `share()` method reads naturally: "share this value with the closure."

### Why not just make cloning automatic?

Some applications genuinely need to track where aliases are created—for performance debugging, memory leak investigation, or APIs like `Arc::make_mut`. Making everything automatic would serve high-level apps well but fail the "low-level enough for a kernel" test. Our approach keeps aliases visible while reducing boilerplate. Once this is done, we will evaluate whether to continue with further changes.

### How does this relate to [RFC #3680]?

[RFC #3680] proposed `.use` syntax and a `Use` trait. That work remains available on nightly and informed our thinking. This goal represents a refined direction based on community feedback and deeper exploration. The `Share` trait is similar in spirit to `Use` but with clearer semantics; move expressions address similar problems to `use ||` closures but generalize more naturally.
