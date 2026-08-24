# Arbitrary Self Types

| Metadata              |                                                                                           |
| :--                   | :--                                                                                       |
| Point of contact      | @dingxiangfei2009                                                                         |
| Status                | Accepted                                                                                  |
| What and why          | Let user-defined smart pointers work as method receivers and support `dyn Trait` coercion |
| Roadmap               | Rust for Linux                                                                            |
| Roadmap               | Beyond the `&`                                                                            |
| Tracking issue        | [rust-lang/goals#619]                                                        |
| Other tracking issues | [rust-lang/rust#44874], [rust-lang/rust#146095], [rust-lang/rust#123430]                  |
| Highlight             | Custom pointer types                                                                      |
| Zulip channel         | N/A                                                                                       |
| [lang] champion       | @tmandry                                                                                  |
| [types] champion      | @jackh726                                                                                 |
| [lang-docs] champion  | @traviscross                                                                              |


## Summary

It is possible to create custom smart pointers by implementing the `Deref` trait. However, the standard library types are able to do things user-defined pointers can't. For example, acting as a method receiver (`self` etc.) or turning a concrete type implementing a custom `Trait` to a `dyn Trait`.

These limitations are of particular importance to low-level systems (e.g. Rust for Linux) and cross-language interoperability (e.g. when designing a reference to C++ objects).

We want to stabilize these features to reduce the gap between smart pointers in the standard library and user-defined ones.

These features are all interconnected and depend on one another so we're proposing a single goal that makes user-defined smart pointers on par with the ones in the standard library.

## Motivation

### The status quo

There are two main limitations custom pointers face today.

First, being able to use one as a method receiver. You can implement a method on a type with the first parameter being `Box<self>`, `Rc<self>`, or `Arc<self>` etc. and then call the method directly on the pointer.

The `arbitrary_self_types` language feature allows users to do the same for their pointers:

```rust
pub struct SmartPointer<T> { ... }

impl<T> Receiver for SmartPointer<T> {
    type Target = T;
}

struct Person { ... };

// It is now possible to use `SmartPointer`s as method receivers.
// Previously, only blessed types like `Box` could act like method receivers.
impl Person {
    pub fn biometrics(self: &SmartPointer<Self>) -> &Biometrics {
        ...
    }
}

let person: SmartPointer<Person> = get_data();
// Method calls can now also dispatch `SmartPointer<Person>` to the intended methods.
let _: &Biometrics = person.biometrics();
```

Arbitrary self types do this by implementing the `Receiver` trait. Right now, we have a blanket `Receiver` implementation for any type that implements `Deref`.

We want to remove that coupling and have `Receiver` be a standalone trait. In particular, we would like to keep the door open for the ability to have `Receiver::Target` and `Deref::Target` different (when both are implemented) in the future.

The second limitation is around coercion to a `dyn Trait`. We want to be able to define custom smart pointers that work with trait objects.

It would allow us to do this:

```rust
#[derive(CoercePointee)]
#[repr(transparent)]
pub struct SmartPointer<T> { ... }

impl<T: ?Sized> Deref for SmartPointer<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

trait MyTrait {}

impl MyTrait for i32 {}

fn main() {
    let ptr: MySmartPointer<i32> = MySmartPointer(Box::new(42));

    // This coercion would be an error without the derive.
    let ptr: MySmartPointer<dyn MyTrait> = ptr;
}
```

### What we propose to do about it

For `arbitrary_self_types`:

- Implement the decoupling of the two traits.
- Decide on the fate of the `arbitrary_self_types_pointers` feature, whose design would run against common language principles and may hinder future library and language feature evolution.
- Update the Rust language reference to codify the method resolution rules while considering possible language extensions from "Beyond &amp;."
- Propose a second stabilisation attempt to conclude the project goal work.

For `Deref`/`Receiver` chain:

While we have settled with splitting the coupling of the two traits so that they will affect method resolution independently, we would also like to open a small language experiment by introducing a feature gate `arbitrary_self_types_split_chain`, through which the Rust language users can experiment with diverging target types in `Receiver` and `Deref`. Through this experiment, we will collect the data on the utility of having this feature in the language and potential use cases of this construction.

- Land the implementation [for decoupling the Deref and Receiver traits rust#146095](https://github.com/rust-lang/rust/pull/146095) at the start of the language experiment.
- Engage the community through various channels and users that have interest in or make use of the `Receiver` language feature to collect opinions, when they find a need to enable this new unstable feature.
- Draft a stabilisation or de-RFC report based on the experiment data to finalise the fate of this unstable feature.

For `derive(CoercePointee)`:

The implementation is largely done, but the stabilization is blocked on `arbitrary_self_types`. Once that feature is stabilized, we can proceed with stabilizing `derive(CoercePointee)` too.


### Work items over the next year

#### Arbitrary self types

Let user-defined smart pointers work as method receivers, stabilizing the `arbitrary_self_types` feature.

| Task                                        | Owner(s)          | Notes |
|---------------------------------------------|-------------------|-------|
| Implementation of `Deref` decoupling        | @dingxiangfei2009 |       |
| (de-)RFC raw pointer receivers              | @dingxiangfei2009 |       |
| Update `arbitrary_self_types` documentation | @dingxiangfei2009 |       |
| `arbitrary_self_types` Reference PR         | @dingxiangfei2009 |       |
| `arbitrary_self_types` Stabilization PR     | @dingxiangfei2009 |       |

#### `Deref`/`Receiver` split chain experiment

Experiment with letting `Receiver::Target` and `Deref::Target` diverge, collecting data on utility and use cases.

| Task                                        | Owner(s)          | Notes |
|---------------------------------------------|-------------------|-------|
| `Deref`/`Receiver` Implementation           | @dingxiangfei2009 |       |
| `Deref`/`Receiver` Community engagement     | @dingxiangfei2009 |       |
| `Deref`/`Receiver` Stabilization PR         | @dingxiangfei2009 |       |
| `Deref`/`Receiver` Reference PR             | @dingxiangfei2009 |       |

#### `derive(CoercePointee)`

Support `dyn Trait` coercion for user-defined smart pointers.

| Task                                        | Owner(s)          | Notes |
|---------------------------------------------|-------------------|-------|
| `derive(CoercePointee)` Reference PR        | @Darksonn         | Blocked on `arbitrary_self_types` |
| `derive(CoercePointee)` Stabilization PR    | @Darksonn         | Blocked on `arbitrary_self_types` |


## Team asks

| Team        | Support level | Notes                      |
|-------------|---------------|----------------------------|
| [lang]      | Medium        | Reviews, Lang/RfL meetings |
| [lang-docs] | Medium        | Reviews, Lang/RfL meetings |
| [libs]      | Small         | Reviews, Stabilizations    |
| [types]     | Large         | Review of type-system stabilization/implementation |

## Frequently asked questions

### What do I do with this space?

*This is a good place to elaborate on your reasoning above -- for example, why did you put the design axioms in the order that you did? It's also a good place to put the answers to any questions that come up during discussion. The expectation is that this FAQ section will grow as the goal is discussed and eventually should contain a complete summary of the points raised along the way.*
