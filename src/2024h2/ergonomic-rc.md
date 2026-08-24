# Ergonomic ref-counting

| Metadata       |                                    |
| ---            | ---                                |
| Point of contact | @jkelleyrtp                        |
| Status         | Accepted                           |
| Tracking issue | [rust-lang/goals#107] |
| Zulip channel  | N/A                                |
## Summary

Deliver nightly support some solution to reduce the ergonomic pain of working with ref-counted and cheaply cloneable types.

## Motivation

For 2024H2 we propose to improve ergonomics of working with "cheaply cloneable" data, most commonly reference-counted values (`Rc` or `Arc`). Like many ergonomic issues, these impact all users, but the impact is particularly severe for newer Rust users, who have not yet learned the workarounds, or those doing higher-level development, where the ergonomics of Rust are being compared against garbage-collected languages like Python, TypeScript, or Swift.

### The status quo

Many Rust applications&mdash;particularly those in higher-level domains&mdash;use reference-counted values to pass around core bits of context that are widely used throughout the program. Reference-counted values have the convenient property that they can be cloned in O(1) time and that these clones are indistinguishable from one another (for example, two handles to a `Arc<AtomicInteger>` both refer to the same counter). There are also a number of data structures found in the stdlib and ecosystem, such as the [persistent collections found in the `im` crate](https://crates.io/crates/im) or the [`Sender` type from `std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html) and [`tokio::sync::mpsc`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Sender.html), that share this same property.

Rust's current rules mean that passing around values of these types must be done explicitly, with a call to `clone`. Transforming common assignments like `x = y` to `x = y.clone()` can be tedious but is relatively easy. However, this becomes a much bigger burden with closures, especially `move` closures (which are common when spawning threads or async tasks). For example, the following closure will consume the `state` handle, disallowing it from being used in later closures:

```rust
let state = Arc::new(some_state);
tokio::spawn(async move { /* code using `state` */ });
```

This scenario can be quite confusing for new users (see e.g. this [2014 talk at StrangeLoop](https://youtu.be/U3upi-y2pCk?si=kFEhRB_O_wdMKysC&t=807) where an experienced developer describes how confusing they found this to be). Many users settle on a workaround where they first clone the variable into a fresh local with a new name, such as:

```rust
let state = Arc::new(some_state);

let _state = state.clone();
tokio::spawn(async move { /*code using `_state` */ });

let _state = state.clone();
tokio::spawn(async move { /*code using `_state` */ });
```

Others adopt a slightly different pattern leveraging local variable shadowing:

```rust
let state = Arc::new(some_state);

tokio::spawn({
    let state = state.clone();
    async move { /*code using `state`*/ }
});
```

Whichever pattern users adopt, explicit clones of reference counted values leads to significant accidental complexity for many applications. As noted, cloning these values is both cheap at runtime and has zero semantic importance, since each clone is as good as the other. 

#### Impact on new users and high-level domains

The impact of this kind of friction can be severe. While experienced users have learned the workaround and consider this to be a papercut, new users can find this kind of change bewildering and a total blocker. The impact is also particularly severe on projects attempting to use Rust in domains traditionally considered "high-level" (e.g., app/game/web development, data science, scientific computing). Rust's strengths have made it a popular choice for building underlying frameworks and libraries that perform reliably and with high performance. However, thanks in large part to these kind of smaller, papercut issues, it is not a great choice for **consumption** of these libraries

Users in higher-level domains are accustomed to the ergonomics of Python or TypeScript, and hence ergonomic friction can make Rust a non-starter. Those users that stick with Rust long enough to learn the workarounds, however, often find significant value in its emphasis on reliability and long-term maintenance (not to mention performance). Small changes like avoiding explicit clones for reference-counted data can both help to make Rust more appealing in these domains **and** help Rust in other domains where it is already widespead. 

### The next six months

The goal for the next six months is to 

* author and accept an RFC that reduces the burden of working with clone, particularly around closures
* land a prototype nightly implementation.

### The "shiny future" we are working towards

This goal is scoped around reducing (or eliminating entirely) the need for explicit clones for reference-counted data. See the [FAQ](#frequently-asked-questions) for other potential future work that we are not asking the teams to agree upon now.

## Design consensus points

We don't have consensus around a full set of "design axioms" for this design, but we do have alignment around the following basic points:

* Explicit ref-counting is a major ergonomic pain point impacting both high- and low-level, performance oriented code.
* The worst ergonomic pain arises around closures that need to clone their upvars.
* Some code will want the ability to precisely track reference count increments.
* The design should allow user-defined types to "opt-in" to the lightweight cloning behavior.

## Ownership and team asks

The work here is proposed by @jkelleyrtp on behalf of Dioxus Labs. We have funding for 1-2 engineers depending on the scope of work. Dioxus Labs is willing to take ownership and commit funding to solve these problems.

| Task                       | Owner(s) or team(s)         | Notes               |
| -------------------------- | --------------------------- | ------------------- |
| Overall program management | @jkelleyrtp                 |                     |
| Author RFC                 | TBD                         |                     |
| Design meeting             | ![Team][] [lang]            | 2 meetings expected |
| RFC decision               | ![Team][] [lang] [libs]     |                     |
| Nightly implementation     | @spastorino                 |                     |
| Standard reviews           | ![Team]  [compiler]         |                     |
| Blog post on Inside Rust   |                             |                     |

* The ![Team][] badge indicates a requirement where Team support is needed.

[Not funded]: https://img.shields.io/badge/Not%20yet%20funded-red
[Approved]: https://img.shields.io/badge/Approved-green
[Not approved]: https://img.shields.io/badge/Not%20yet%20approved-red
[Complete]: https://img.shields.io/badge/Complete-green
[TBD]: https://img.shields.io/badge/TBD-red
[Team]: https://img.shields.io/badge/Team%20ask-red

### Support needed from the project

As owners of this goal...

- We are happy to author RFCs and/or work with other experienced RFC authors.
- We are happy to host design meetings, facilitate work streams, logistics, and any other administration required to execute. Some subgoals proposed might be contentious or take longer than this goals period, and we're committed to timelines beyond six months.
- We are happy to author code or fund the work for an experienced Rustlang contributor to do the implementation. For the language goals, we expect more design required than actual implementation. For cargo-related goals, we expected more engineering required than design. We are also happy to back any existing efforts as there is ongoing work in cargo itself to add various types of caching.
- We would be excited to write blog posts about this effort. This goals program is a great avenue for us to get more corporate support and see more Rust adoption for higher-level paradigms. Having a blog post talking about this work would be a significant step in changing the perception of Rust for use in high-level codebases.

The primary project support will be design bandwidth from the [lang team].

## Outputs and milestones

### Outputs

*Final outputs that will be produced*

### Milestones

*Milestones you will reach along the way*

## Frequently asked questions

[jkelleyrtp]: https://github.com/jkelleyrtp
[spastorino]: https://github.com/spastorino

### After this, are we done? Will high-level Rust be great?

Accepting this goal only implies alignment around reducing (or eliminating entirely) the need for explicit clones for reference-counted data. For people attempting to use Rust as part of higher-level frameworks like Dioxus, this is an important step, but one that would hopefully be followed by further ergonomics work. Examples of language changes that would be helpful are described in the (not accepted) goals around a renewed [ergonomics initiative](./ergonomics-initiative.md) and [improve compilation speed](./faster-iterative-builds.md).