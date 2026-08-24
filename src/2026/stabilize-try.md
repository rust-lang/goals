# Stabilize the Try trait

| Metadata              |                                    |
| :--                   | :--                                |
| Point of contact      | @MusicalNinjaDad                   |
| Status                | Accepted                           |
| Other tracking issues | [rust-lang/rust#84277]             |
| Zulip channel         | [#t-lang/try][channel]             |
| Highlight             | Try, never, extern types           |
| [lang] champion       | @tmandry                           |
| [libs-api] champion   | @Amanieu                           |
| Tracking issue        | [rust-lang/goals#654] |

[channel]: https://rust-lang.zulipchat.com/#narrow/channel/605325-t-lang.2Ftry

## Summary

Stabilize the `Try` trait, which customizes the behavior of the `?` operator.

## Motivation

### The status quo

Today the only types that can be used with the `?` are in the standard library. Most commonly these are `Option<T>` and `Result<T, E>`. While most Rust code is happy using one of these two, there are use cases not addressed by these types.

#### Capturing error context

A common use case is capturing the context of an error each time it is bubbled up using `?`, without resorting to the use of backtraces.

```rust
enum TracedResult<T, E> {
    Err(TracedError<E>),
    Ok(T),
}
use TracedResult::Ok;

fn read_list(path: PathBuf) -> TracedResult<Vec<i32>> {
    let file = File::open(path)?;
    Ok(read_number_list(file)?)
}

fn read_number_list(file: File) -> TracedResult<Vec<i32>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    ...
}

fn main() -> TracedResult<()> {
    let list = read_list("path/to/file.txt".into())?;
    println!("{list:?}");
}

impl<T, E> FromResidual<TracedResult<!, E>> for TracedResult<T, E> {
    #[track_caller]
    fn from_residual(residual: TracedResult<!, E>) -> Self {
        let location = Location::caller();
        match residual {
            TracedResult::Err(err) => TracedResult::Err(err.with_source_location(location))
        }
    }
}

impl<T, E> FromResidual<Result<!, E>> for TracedResult<T, E> {
    #[track_caller]
    fn from_residual(residual: Result<!, E>) -> Self {
        let location = Location::caller();
        match residual {
            Result::Err(err) => TracedResult::Err(TracedError::new(err, location))
        }
    }
}
```

Could result in the output:

```
Error: Failed to read file contents
at program.rs:19:48
at program.rs:9:27
at program.rs:14:29
```

While this is not a user-friendly error message, some errors like internal server errors are never shown to users. In those cases it is reasonable to show only a backtrace for developers.

Capturing a backtrace at the time of initial error creation is prohibitively expensive, both because unwinding is slow and because errors are often handled higher up in the stack without being printed. Capturing a single `Location<'static>` pointer for each level of bubbling up can be optimized much better.

Because this functionality is so critical for Rust bindings to the [Abseil Status](https://abseil.io/docs/cpp/guides/status) library, their plan is to use a custom `try_status!()` macro to capture the location information instead of `?`. This imposes a heavy burden on users, much like the `try!()` macro did before it.

### What we propose to do about it

Stabilize the `Try` trait, which has been in its current form since 2021.

Because there are open design questions that haven't yet been resolved, we should either

* Find a way to resolve them, or
* Stabilize in a limited way, unblocking the basic use case while leaving our options open for the future.

### Work items over the next year

| Task                                                               | Owner(s) | Notes |
| ------------------------------------------------------------------ | -------- | ----- |
| Design and implement answers to the remaining unresolved questions |          |       |
| Lead design discussion with lang and libs-api teams                |          |       |
| Write and shepherd a stabilization report                          |          |       |

## Team asks

| Team       | Support level | Notes                                   |
| ---------- | ------------- | --------------------------------------- |
| [compiler] | Small         |                                         |
| [lang]     | Medium        |                                         |
| [libs-api] | Medium        |                                         |
| [types]    | Small         |                                         |

## Funding

| Purpose | Cost | Funded | Sponsor(s) |
|---------|------|--------|------------|
| Contributor | TBD | Partial | |

## Frequently asked questions
