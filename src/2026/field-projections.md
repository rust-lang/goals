# Field Projections

| Metadata             |                                           |
| :------------------- | ----------------------------------------- |
| Contact     | @BennoLossin                              |
| Status               | Accepted                                  |
| What and why         | Access fields through smart pointers and pinned references, not just `&` and `&mut` |
| Timespan             | 2026-2028                                 |
| Roadmap              | Beyond the `&`                            |
| Roadmap              | Rust for Linux                            |
| Tracking issue       | [rust-lang/goals#390]        |
| Highlight            | Custom pointer types                      |
| Zulip channel        | [t-lang/custom-refs][custom-refs-channel] |
| [lang] champion      | @tmandry                                  |
| [types] champion     | @nikomatsakis                             |
| [compiler] champion  | @dingxiangfei2009                         |
| [opsem] champion     | @digama0                                  |

[custom-refs-channel]: https://rust-lang.zulipchat.com/#narrow/channel/522311-t-lang.2Fcustom-refs

## Summary

We aim to explore and refine the *virtual places* approach for field projections, document its design and interactions in the [beyond-refs wiki](https://rust-lang.github.io/beyond-refs/), implement it as an experiment in the compiler, and prepare RFCs based on the findings.

This is a continuing goal, see [the goal document of the previous period](https://goals.rust-lang.org/2025h2/field-projections.html) for historical information.

## Motivation

Field projections aim to provide a uniform and ergonomic way to produce pointers to fields for custom pointer types. This mirrors the operation on built-in references available today: `&x.field` is of type `&Field` given that `x: &Struct`. This project goal aims to make this feature generally available for all custom types and implement it for `&mut MaybeUninit<T>`, `NonNull<T>` and many more.

This feature will reduce verbosity and increase ergonomics when working with custom pointer types. It also enhances expressiveness, as it provides native borrow checker integration and allows the simultaneous usage of pointers to disjoint fields of the same struct.

### The status quo

There are many examples for types that can take advantage of field projections. In its current form the design is a generalization of `Deref` that provides an umbrella abstraction for *pointers to virtual places*. As the name suggests, these places do not really need to exist, so `struct MyStruct<T>(PhantomData<T>)` is supported by our approach. Naturally any type that implements `Deref` is supported; but also types that cannot implement it, such as raw pointers, `NonNull<T>` and many more are covered.

In the previous goal period, we held a [design meeting](https://hackmd.io/@rust-lang-team/S1I1aEc_lx) about the general approach to designing a solution. In it we also extensively covered many use-cases. Here is a non-exhaustive list of types that would benefit from this feature:
- `&mut MaybeUninit<T>`,
- `cell::Ref[Mut]<'a, T>`
- `NonNull<T>` and `*{const,mut} T`
- `pyo3::pycell::PyRef[Mut]<'_, T>`
- `Cpp[Mut]Ref<T>`
- `&[mut] Cell<T>`
- `&Atomic<T>`
- `ArcRef<T>`
- `VolatilePtr<'_, T>`
- `&[mut] Untrusted<T>`
- `SeqLockRef<'_, T>`
- `UserPtr<T>`
- `Ptr<'_, T>`

We also want to note that another contributor separately (without being aware of the field projection effort) created an [ACP for `ArcRef<T>`](https://github.com/rust-lang/libs-team/issues/700). Another [wanted to introduce swift's keypath feature](https://rust-lang.zulipchat.com/#narrow/channel/213817-t-lang/topic/Fields.20projection.20with.20Keypaths/with/567490323) to Rust, which is very similar to a part of field projection.

### What we propose to do about it

The last goal period resulted in a new approach for field projections called *virtual places*. It allows customizing *place operations* via traits: `PlaceRead`, `PlaceWrite`, `PlaceMove`, and most importantly `PlaceBorrow`. We are missing some interactions, concrete details and a comprehensive document on this design, but the overall idea is solid. It is also much too complicated for a project goal. As part of the goal, we are writing a [wiki](https://rust-lang.github.io/beyond-refs/) to better explain all of the interactions with other Rust features.

Here is an example on how the current approach could look like for improving the ergonomics of `NonNull`:

```rust
struct Struct {
    field: Field,
}

struct Field {
    accesses: usize,
}

impl Struct {
    unsafe fn reset(this: NonNull<Self>) {
        // We can borrow using `NonNull` directly:
        let field: NonNull<Field> = unsafe { @NonNull (*this).field };
        // Note that we can omit the dereference.
        // We can also use the canonical borrow, which fills `NonNull` for us:
        let field: NonNull<Field> = unsafe { @this.field };

        // We can also borrow using a totally different pointer if that is
        // supported by the underlying type:
        unsafe { std::ptr::write(@raw mut (*field).accesses, 0) };

        // Alternatively, we could also just have written to the field directly:
        unsafe { field.accesses = 0 };
        // Note that this drops the previous value, which does nothing for `usize`.
    }
}
```

Field projections is not limited to low-level constructs like raw pointers. It can also help with smart pointers like `ArcRef<T>`: `ArcRef<T>` is like an `Arc<T>`, but with separate pointers to the data and the refcount. This allows us to offset the data pointer and produce `ArcRef<Field>` from `ArcRef<Struct>`.

```rust
impl Struct {
    fn get_accesses(self: ArcRef<Self>) -> ArcRef<usize> {
        // Again, we use the canonical borrow. Also note that we can
        // go through any number of fields in one borrow operation.
        @self.field.accesses
    }
}
```

The design axioms from the last period still apply and are fulfilled by the virtual places approach:

- **Effortless Syntax.** Using field projections in a non-generic context should look very similar to normal field accesses.
- **Broad Solution.** Field projections should be very general and solve complex projection problems such as pin-projections and [`RcuMutex<T>`](https://hackmd.io/@rust-lang-team/S1I1aEc_lx#RCU-Read-Copy-Update).

### Work items over the next year

#### Map the field projection design space

Explore the virtual places approach, document it in the beyond-refs wiki, formalize borrow checker integration, and build a compiler experiment.

| Task        | Owner(s) | Notes |
| ----------- | -------- | ----- |
| Establish a working group for field projections | @tmandry | The group should start out with the members: @BennoLossin, @Nadrieril, @tmandry, @dingxiangfei2009. Any contributor is welcome to join if they intend to develop and design field projections. |
| Explore and map the solution space | field projection working group | Discussions in [t-lang/custom-refs](https://rust-lang.zulipchat.com/#narrow/channel/522311-t-lang.2Fcustom-refs) & meetings with members of the working group  |
| Document the design in the wiki | @BennoLossin | Write it down in RFC-style, to easily extract RFCs or design meeting documents from the wiki. |
| Formalize the borrow checker integration in a-mir-formality | @BennoLossin, @nikomatsakis | Verify our work formally and explore the algorithms needed for implementing it in the compiler |
| Implement a compiler experiment | @BennoLossin, @dingxiangfei2009, @oli-obk | Evaluate our current approach by creating an experiment to try out in real code. Implementers: @BennoLossin & @dingxiangfei2009; Reviewer: @oli-obk |
| Draft RFCs | @tmandry, @BennoLossin | Extract the knowledge from the wiki & provide historical context as well as rationale and a contiguous & comprehensive story. |

**Success metric:** This project goal will be successful if it can significantly advance the design and knowledge on how to implement field projections in Rust; part of that is creating a compiler experiment in nightly. If we are able to accept the required RFCs, then we have over-achieved our goal. A major setback would be if we discover the current approach untenable or find other blockers that prevent making meaningful progress in the design and experiment.

### The "shiny future" we are working towards

Field projections is part of a larger idea called *beyond references*. There should be no built-in types in the Rust language that a library could not recreate. For example, a user should be able to implement a `MyBox<T>` that allows moving in and out, supports unsizing, coercions and borrowing the contents using references. Field projections enable that last feature as well as the moving out support. Ultimately, using any library declared type should feel as if it was built into the language itself. In this future, no significant compiler magic exists for references and they can be fully implemented in `core`.

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [lang]     | Large         | Aiming for two design meetings; large language feature |
| [compiler] | Medium        | Reviews of big changes needed; also looking for implementation help |
| [types]    | Medium        | Collaborating on a-mir-formality on the borrow checker integration; small reviews of RFC and/or compiler PRs |
| [libs]     | Small         | Small reviews of library PRs (implementing FP for core & std types) and RFC |
| [opsem]    | Small         | Small reviews of RFC and/or compiler PRs |

The lang team support level is Large, since we could end up with an RFC review and decision at the end of the year. If not, then the input from t-lang is much more manageable and more in-line with Medium: the design meetings and the champion suffice as support.

## Help wanted

| Task | Experience level | Time investment |
|------|-----------------|-----------------|
| Fix a [bug in rustdoc](https://github.com/rust-lang/rust/issues/158768) ([context](https://github.com/rust-lang/goals/issues/390#issuecomment-4881394710)) | Familiarity with rustdoc and procedural macros |  |

## Frequently asked questions
