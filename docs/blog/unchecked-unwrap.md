---
title: use unchecked_unwrap in rust to go fast
description: This blog post explains how we can leverage unchecked_unwrap to gain performance.
slug: unchecked-unwrap-in-rust
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [rust]
image: /img/running.jpg
hide_table_of_contents: false
---

![running](/img/running.jpg)


Rust is well-known for its performance and memory safety. This is one of the primary reasons we chose rust to build the inspektor's dataplane. So that we can extract as much performance as we can from the machine.

`unwrap` is a method in the `Option` enum that most rustaceans are familiar with. The `unwrap` function will determine whether the `Option` enum is `None` or `Some`. If the variant is `Some`, then returns the value. Otherwise, it throws panic.

Here is the implementation of `unwrap` function.

```rust
 pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
        }
    }

```

To avoid pattern matching, it's common practice to use `unwrap` when the developer knows the `Option` enum is of the `Some` variant.

```rust
fn main() {
    let list = vec![1,3];
    println!("{}", list.get(1).unwrap())
}
```

`unwrap` can be replaced with `unchecked_unwrap` to get optimized code. 

```rust
fn main() {
    let list = vec![1,3];
    unsafe{
        println!("{}", list.get(1).unwrap_unchecked());
    }
}
```

The only catch is that it is an unsafe function, and no backtrace will be shown if the code crashes.

Here is a failure case with `unwrap` and `unchecked_unwrap` respectively

**with unwrap**
```shell
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/main.rs:4:32
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
``` 

**with unchecked_unwrap**
```shell
Illegal instruction (core dumped)
```

The `unchecked_unwarp` is optimized because there is no panic handler to handle program crashes. 

Here is the assembly code and benchmark of `unwrap` and `unchecked_unwarp` from the [rust docs](https://docs.rs/unchecked_unwrap/latest/unchecked_unwrap/). 

**unwrap assembly code** 
```asm
push    rax
test    rdi, rdi
je      .LBB2_1       // panic handler
mov     rdx, rsi
mov     rax, rdi
pop     rcx
ret
```
**unchecked_unwap assembly code**

```asm
mov     rdx, rsi
mov     rax, rdi
ret
``` 

**benchmarks**

```shell
test checked::expect_option   ... bench:         798 ns/iter (+/- 90)
test checked::expect_result   ... bench:         724 ns/iter (+/- 109)
test checked::unwrap_option   ... bench:         802 ns/iter (+/- 52)
test checked::unwrap_result   ... bench:         743 ns/iter (+/- 176)
test unchecked::expect_option ... bench:         407 ns/iter (+/- 93)
test unchecked::expect_result ... bench:         374 ns/iter (+/- 48)
test unchecked::unwrap_option ... bench:         345 ns/iter (+/- 53)
test unchecked::unwrap_result ... bench:         407 ns/iter (+/- 22)
``` 

*So, if you know what you are doing then `unchecked_unwrap` would you give you a nice performance bump.*
 
