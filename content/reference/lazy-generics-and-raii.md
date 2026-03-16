---
title: "Traits, generics and ownership"
description: "Here's what Lazy can teach us about two of the key concepts in the Rust type system."
---

## Lazy and traits

Here is the struct definition of `Lazy`:

```rust
struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>
}
```

_"Wait, so how does `Lazy` 'know' when to read the value?"_, I hear you ask. Good question! The answer is that `Lazy` "knows" when to initialize the value because it implements Rust's Deref trait.

Here's what happens under the hood:

- Whenever you try to use an instance of `Lazy` in your code (by calling a method, reading a field, dereferencing it explicitly with the `*` operator, like `*this`), Rust automatically calls the `deref()` method on the `Lazy` struct.
- Inside that `deref()` implementation, `Lazy` calls `Lazy::force()`. `force` is a method that basically just calls our closure (more-or-less equivalent to an arrow function from JS-land or a lambda from Python). It doesn't mean "recalculate it forcefully" but rather "force this lazy value to resolve itself _now!_"
- `force()` checks its internal `OnceCell` (`cell` field). If the cell is empty, it extracts and runs the closure we passed in, stores the result inside the cell, and returns the reference.
- And finally, if it has already been initialised on a previous access, it completely skips the closure and just returns a reference to the data inside the cell.

## Lazy and Generics

Another interesting thing `Lazy` can teach us is how generics work. If you remember Week 1, we used a vector (Vec), which used the "T" generic. Now let's look at `Lazy`'s struct definition again to see how it uses generics:

```rust
struct Lazy<T, F = fn() -> T> {
  cell: OnceCell<T>,
  init: Cell<Option<F>>,
}
```

We are using generics in a couple ways here:

- Firstly, we're constraining Lazy's type so that the type returned from `Lazy::new`'s closure is the type that's going to be kept inside our `Lazy`. So for instance, the below code:

  ```rust
    Lazy::new(|| {
      envy::from_env::<Config>()
        .wrap_err("failed to load config")
        .unwrap()
    });
  ```

  is going to return us a `Lazy<Config>` because the closure (as denoted by the `||`) always returns a `Config` (remember that functions and closures can only return one type - even if it's 'split'[1] with an enum or struct, that still counts as one type).
- We're constraining the closure with `fn() -> T`. This means that if the type in the static is defined as a `Lazy<String>` but we return e.g. an `i32` from the static, we'll run into an error (same thing as above). But it also means that the closure will always take in zero arguments, because `fn()` means that we want a function that doesn't take any arguments.
- `OnceCell` also uses a generic to know what to store inside it! We pass in the generic from our `Lazy` down to the `OnceCell`, so a `Lazy<Config>` will also have a `OnceCell<Config>` inside it. We also use a `Cell` that keeps our closure.

## Wait, why the `Cell<Option<F>>`?

Two things:

- We need to be able to take ownership in order to be able to call a passed-in closure (`F`). `OnceCell` doesn't let us do that since it needs to keep the data in memory for other readers to be able to use it.
- We can use an `Option<F>` to be able to take the closure, call it, and then no longer store the closure! This saves us a bit of memory since the closure then gets dropped due to RAII.

## "RAII?"

RAII stands for **Resource Acquisition Is Initialization**. Basically, once a value is dropped, the destructor (function that handles cleanup) is called, and the data from that value is freed from memory. This has two big advantages:

- We don't have to run into use-after-free bugs like in languages like C, but we also don't just keep the memory around forever. So once it's not needed, it's removed without us having to think about it. And at the same time, we don't need to use a [garbage collector](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Memory_management) like in JavaScript either.
- Unlike in some other languages, you don't need to worry about things like closing files (e.g. `file.close()` in Python, unless you open the file using a `with` block), because, if implemented, the destructor can do things like that for you! This is something that the standard library does for you.

A beautiful way that RAII is used is the `drop` function. If you ever wanted to drop a value before the function returns for some reason, you can use the `drop` function in the standard library to do it. Here's the entire implementation of `drop`:

```rust
fn drop<T>(_: T) {}
```

...that's it! The generic T allows us to use any value at all with our drop function, and because we take ownership of the value (`_`) and don't return it, it now has no owner. That means that the memory is automatically freed!

To implement a destructor, we can implement the `Drop` trait:

```rust
struct HasDrop;

impl Drop for HasDrop {
    fn drop(&mut self) {
        println!("Dropping HasDrop!");
    }
}
```

And the memory will be freed for us after `println!`.
