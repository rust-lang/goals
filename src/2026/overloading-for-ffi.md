# Nightly support for function overloading in FFI bindings

| Metadata         |                                    |
| :--              | :--                                |
| Point of contact | @teor2345                          |
| Status           | Accepted                           |
| Tracking issue   | [rust-lang/goals#643] |
| Zulip channel    | N/A                                |
| [lang] champion  | @tmandry                           |


## Summary

Design and implement a function overloading system that is suitable for FFI with languages that have overloading. In particular, we aim to make Rust more compatible with C++, so that Rust callers do not place a maintenance burden on C++ library owners due to overloading.

## Motivation

### The status quo

#### Overloading on arguments

C++ offers the ability to define and call **overloaded functions** (also called "overload sets"): sets of functions with the same name, which expose multiple function signatures. These can dispatch both on function arity, and on the types for functions with the same number of parameters, selecting based on both the method receiver type and the ordinary method or function argument types. Calls select the correct overload using a name resolution system that takes the function signature into account. If these functions are to be called from Rust, we need some mechanism to call them.

Rust already has overloading, in the form of trait dispatch. For example, the overloaded C++ constructor `MyType(int32_t); MyType(int32_t, int32_t)` can become impls of something like the Rust traits `From<(i32,)>` and `From<(i32, i32)>`.

Using Rust's existing trait system is syntactically difficult. In order to support overloading on arity, we must use an auxiliary type for the list of arguments, such as a tuple. This means that calls like `x.foo()` and `x.foo(1, 2)` must become `x.foo(())` and `x.foo((1, 2))`. ([playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=bb70f36ac88ec8df4875c37e0161baff))

```rust
impl Foo<()> for MyType {
    type Return = i32;
    fn foo(&self, _args: ()) -> i32 {
        42
    }
}
impl Foo<(i32, i32)> for MyType {
    type Return = i32;
    fn foo(&self, args: (i32, i32)) -> i32 {
        args.0 + args.1
    }
}

x.foo(());
x.foo((1, 2));
```

This presents an obvious ergonomic problem. And _all_ methods must look like this, or else there is a code maintenance and compatibility hazard: adding an overload to C++ would break Rust callers, but not equivalent C++ callers.

Secondly, Rust's trait coherence rules severely limit the set of possible overloads compared to C++, which does not have any version of coherence. (See FAQ for details.)

At the time of writing, the only fully general solution for overloading is to manually rename each overload in the overload set, but this means that existing C++ interfaces are not callable from Rust automatically, and can mean that adding an overload can change whether existing functions get bindings. Ideally, adding or removing an overload should not be more backwards-incompatible for C++ than for Rust, and supporting Rust callers should not be a higher burden than supporting C++ callers.

#### Overloading on receivers

Consider the `clear` method on a C++ container type. If you have a unique reference to the container, this is safe, but if you have an aliasing reference – even if it is otherwise safe to call functions accepting aliasing references – this function would be unsafe.

```rust
pub fn clear(&mut self) {self.size = 0; free(self.data)}
pub unsafe fn clear(self: CMut<'_, T>) {unsafe {CMut::as_mut(self).clear();}
```

Rust and C++ have many reference types (potentially something like `&T`, `&mut T`, `Pin<&mut T>`, `CRef<T>`, `CMut<T>`, `RvalueRef<T>`, `ConstRvalueRef<T>`, `CRvalueRef<T>` and `CConstRvalueRef<T>` (for aliasing/safety discrimination)). It will sometimes be necessary to implement the same method for all nine or so of them: either because in C++, they are separate overloads with separate implementations, or because they need separate safety or other function properties.

There are only a finite number of reasonable receivers, so we could generate a function per receiver type, using different suffixes like `_pin` or `_cmut`. But if we can overload on the receiver, these could be compressed into an API which is much easier to work with as a human.

### What we propose to do about it

We would like to implement a form of function overloading which is sufficient to seamlessly call C++ code. As part of this, there will be significant design work. Ideally, by the end of this year, there is a set of unstable features which can be trialed by existing FFI code generators to see where the pain points are, for further iteration in 2027 and onward.

Particular approaches we intend to explore:

- Leveraging the existing trait system to perform something which is syntactically similar to C++ overloading. (For example, by extending `extern "rust-call"`, or some other form of splatting.)
- Integrate with existing or proposed unstable features, such as specialization, to cover as much of the C++ overload space as possible.

As an example, something like:

```rust
trait MethodArgs : Tuple {
    fn call_method(self, this: &Foo);
}

impl Foo {
    fn method(&self, #[splat] args: impl MethodArgs) { args.call_method(self) }
}

impl MethodArgs for (i32, String) {
    fn call_method(self, this: &Foo) { dbg!(self.0) }
}
```

#### Design Axioms

- Be comprehensive. Overloading should not be a reason that a C++ function cannot be called.
- Preserve maintainability across the language boundary. Adding or removing an overload should not be substantially more backwards-incompatible in Rust than it is in C++. Supporting Rust callers should not require more work than supporting C++ callers.
- Keep Rust nice. Build on existing concepts where possible: a natural _extension_ of existing language semantics, not a _replacement_ for them.
- Avoid surprises. If a function call compiles, it should pick the overload you most expect.

It is **not** a goal to match C++'s resolution rules exactly. Many languages implement overloading, and Rust may want to interoperate with more than one of them, even if they have conflicting resolution rules. We will not design C++'s [argument-dependent lookup (ADL)](https://en.cppreference.com/w/cpp/language/adl.html) for Rust. Every C++ function should be callable, but that does not mean that it has to be callable in exactly the same way with the same arguments. It is OK to require explicit conversions or markers to select an overload.

### Work items over the next year

| Task                                                | Owner(s)           | Notes                       |
| --------------------------------------------------- | ------------------ | --------------------------- |
| Design an overloading mechanism that is fit for FFI | @ssbr              | @tmandry to act as champion |
| Implement it as a lang experiment                   | @teor2345          | @tmandry to act as champion |

## Team asks

| Team    | Support level | Notes                                                                                                                                  |
| ------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| [lang]  | Medium         | Design meeting Experiment                                                                                                              |
| [libs]  | Small         | Would like to know if they have use cases for overloading in standard Rust, or if there are certain approaches they would like better. May be involved if experiment involves library surface area (e.g. `Fn` traits) |
| [types] | Small        | No dedicated reviewer needed/given, but tracking issue should note the needed for dedicated types review prior to stabilization |
| [compiler] | Small        | Most complexity is in the type system                                                                                                |


## Frequently asked questions

### What do existing tools do with overloads?

[Crubit](http://crubit.rs): overloaded constructors and operators become trait impls. Other overloaded functions do not receive bindings unless the function has a unique Rust name specified using an attribute.

[cxx](https://cxx.rs/): overloaded functions do not receive bindings.

[autocxx](https://google.github.io/autocxx/cpp_functions.htm) and bindgen: overloaded functions receive a numbered name: `func1`, `func2`, etc.

### Incoherent overload sets

The following overload set, in C++, is perfectly valid:

```c
struct ConvertibleFromInt {
    ConvertibleFromInt(int);
};
void Foo(ConvertibleFromInt);
void Foo(int);
```

The first overload accepts anything which implicitly converts to a `ConvertibleFromInt`, while the second overload accepts anything which implicitly converts to an `int`. These overlap: some types implicitly convert to _both_, including `int` itself, as well as `char` and other integral types. C++ evaluates which overload to select using a [multi-step process](https://en.cppreference.com/w/cpp/language/overload_resolution.html), but the gist is that it builds up a set of candidates (two in this case), and then selects the "best viable function" from those.

This is unlike Rust's trait system, which fails compilation at the trait implementation if it can overlap, even if the overlap is only in theory for a type that does not exist, and even if one implementation is in some sense a better match. It is not _possible_ to write a trait that models the above overload set for `Foo`. The `Foo(ConvertibleFromInt)` overload should apply to all types that implement something like `Into<ConvertibleFromInt>`, while the `Foo(int)` overload should take types that implement something like to `Into<c_int>`. Rust cannot prove these are disjoint, because they are not, and overlapping impls of the form `impl<T: Into<c_int>> Foo<T> for …` will fail to compile. ([playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=b6a131d2544aa927dcf81a39a49e482a))

Similarly, it is not possible to implement an overload set that accepts multiple [`impl PinInit` arguments for in-place initialization](https://rust-lang.github.io/goals/2025h2/in-place-initialization.html) of non-rust-movable arguments, even if the `PinInit` trait used an associated type to try to disambiguate (unless/until traits get [disjoint associated types](https://github.com/rust-lang/rust/issues/20400)).

### What about `extern "rust-call"`?

Rust has an existing way to pack function arguments into a tuple as part of ordinary call syntax: `extern "rust-call"`, in the `Fn*` family of traits. For example, the following unit struct acts like an overloaded top-level function:

```rust
struct TopLevelFunc;

impl FnOnce<(u32,)> for TopLevelFunc {
    type Output = ();
    extern "rust-call" fn call_once(self, _args: (u32,)) -> () {}
}
impl FnOnce<(u32, u32, u32)> for TopLevelFunc {
    type Output = ();
    extern "rust-call" fn call_once(self, _args: (u32, u32, u32)) -> () {}
}
TopLevelFunc(1);
TopLevelFunc(1, 2, 3);

```

([playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=8cc3b10155a988d49a852dd2cacb5449))

However, this is less a feature of `rust-call` and more a feature of `Fn*`. If you directly define a method using `extern "rust-call"`, it still must be called using tuple arguments, not unpacked arguments as with the `Fn*` traits.

You might imagine implementing method overloading using `extern "rust-call"` as so:

```rust
trait Method<Args: Tuple> { extern "rust-call" fn method(&self, args: Args); }

struct MyType;
impl Method<(u32,)> for MyType { extern "rust-call" fn method(&self, args: (u32,)) {} }
impl Method<(u32, u32, u32)> for MyType { extern "rust-call" fn method(&self, args: (u32,)) {} }
```

But this does not allow calls like `MyType.method(1, 2, 3)`. We would need an additional feature for splatting arguments outside of the `Fn*` traits. ([playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=da283136e3ad3a8b0afa9925693f789a))
