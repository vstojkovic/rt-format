# `rt-format`

Fully-runtime equivalent of the `format!` macro.

Allows formatting strings like the `format!` macro, with the formatting string and the arguments
provided at runtime. This crate supports all the formatting features of the `format!` macro,
except for the fill character.

## When (not) to use this crate

There are many crates that can be used for formatting values into strings. Here are some of the
criteria you can use to decide if this crate is the right choice for you:

* Can you specify all the formatting options at compile time? If yes, then 
[`std::fmt`](https://doc.rust-lang.org/std/fmt/) is a better option than this crate. If you need to
supply the formatting specifiers at runtime, then this crate might be a viable option.
* Are you formatting only numbers? If yes, consider 
[`num-runtime-fmt`](https://crates.io/crates/num-runtime-fmt) or 
[`num-format`](https://crates.io/crates/num-format).
* Is using Rust nightly an option? If so, consider
[`runtime-fmt`](https://crates.io/crates/runtime-fmt).
* Do you need the ability to implement new formats? If yes, consider 
[`dynfmt`](https://crates.io/crates/dynfmt).
* Do you need `no-std` support? If so, you need to use one of the other alternatives.
* Is formatting likely to be a performance bottleneck for you? If so, you should consider one of
the other alternatives. At this time, there are no benchmarks to compare the approach in this crate
with other crates.
* Is stable API a must-have? If so, you might consider the alternatives. This crate is still not
at version 1.0, which means that the API is still not completely stable.
