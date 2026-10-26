# Fallible allocation functions for Vec

Fallible allocation functions for the Rust standard library's [`alloc::vec::Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) type.

These functions are designed to be usable with `#![no_std]`, `#[cfg(no_global_oom_handling)]` (see
<https://github.com/rust-lang/rust/pull/84266>) enabled and Allocators (see <https://github.com/rust-lang/wg-allocators>).

By default this crate requires the nightly compiler, but the stable compiler can be used if all
features are disabled (i.e., specifying [`default-features = false` for the dependency](https://doc.rust-lang.org/cargo/reference/features.html#the-default-feature)).

## Usage

The recommended way to add these functions to `Vec` is by adding a `use` declaration for the
`FallibleVec` trait: `use fallible_vec::FallibleVec`:
```rust
use fallible_vec::{FallibleVec, try_vec};

let mut vec = try_vec![1, 2]?;
vec.try_push(3)?;
assert_eq!(vec, [1, 2, 3]);
```

## Panic Safety

These methods are "panic safe", meaning that if a call to external code (e.g., an iterator's
`next()` method or an implementation of `Clone::clone()`) panics, then these methods will leave the
`Vec` in a consistent state:
* `len()` will be less than or equal to `capacity()`.
* Items in `0..len()` will only be items originally in the `Vec` or items being added to the `Vec`.
  It will never include uninitialized memory, duplicated items or dropped items.
* Items originally (but no longer) in the `Vec` or being added to (but not yet in) the `Vec` may be
  leaked - any method that may leak items like this will have a note to specify its behavior.

The exact behavior of each method is specified in its documentation.

## Code origin

Most of this code is forked from [Rust's Standard Library](https://github.com/rust-lang/rust). While
we will attempt to keep the code and docs in sync, if you notice any issues please check if they
have been fixed in the Standard Library first.

## This API is incomplete

There are many more infallible functions on `Vec` which have not been ported yet. If there's a
particular API that you're missing feel free to open a PR or file an Issue to get it added.

## Why are these not already in the Standard Library?

There was a [PR to add these and more](https://github.com/rust-lang/rust/pull/95051) to the Standard
Library, followed by an [RFC to discuss if it's a good idea or not to do so](https://github.com/rust-lang/rfcs/pull/3271).
These were closed with the hopes of reopening them once [Keyword Generics](https://blog.rust-lang.org/inside-rust/2022/07/27/keyword-generics.html)
are made available and so "fallible" variants of the existing functions can be added without
exploding the API surface of `Vec`.

## Why would I use this crate versus similar crates?

In general, `fallible_vec` is only useful in situations where `#[cfg(no_global_oom_handling)]` is
required, or if using the Allocator API (functions ending in `_in`). Other crates use APIs that
don't exist when `#[cfg(no_global_oom_handling)]` is enabled (like `vec::push`), whereas
`fallible_vec` reimplements each function to avoid these APIs and builds with `#[cfg(no_global_oom_handling)]`
in its CI.

`fallible_vec` focuses on `vec` alone, whereas other crates provide support for additional types
(like `Box` and `HashMap`).

Comparing `fallible_vec` to [`fallible_collections`](https://crates.io/crates/fallible_collections):

|                                           | `fallible_vec` v0.3.1 | `fallible_collections` v0.4.7 |
|-------------------------------------------|:---------------------:|:-----------------------------:|
| Supports `no_std`                         | X                     | X                             |
| Supports `#[cfg(no_global_oom_handling)]` | X                     |                               |
| Requires nightly rust compiler by default | X                     |                               |
| Supports stable rust compiler             | X                     | X                             |
| `vec::try_append`                         |                       | X                             |
| `vec::try_extend`                         | X                     |                               |
| `vec::try_extend_from_slice`              | X                     | X                             |
| `vec::try_insert`                         | X                     | X                             |
| `vec::try_push`                           | X                     | X                             |
| `vec::try_push_give_back`                 |                       | X                             |
| `vec::try_resize`                         | X                     | X                             |
| `vec::try_resize_with`                    | X                     | X                             |
| `vec::try_splice_in`                      | X                     |                               |
| `try_collect`                             | X                     | X                             |
| `try_collect_in`                          | X                     |                               |
| `try_from_iterator`                       |                       | X                             |
| `try_with_capacity`                       | X                     |                               |
| `try_with_capacity_in`                    | X                     |                               |
| `try_vec!`                                | X                     |                               |
| `try_vec_in!`                             | X                     |                               |
| `Box::*`                                  |                       | X                             |
| `Arc::*`                                  |                       | X                             |
| `Rc::*`                                   |                       | X                             |
| `HashMap::*`                              |                       | X                             |
| `try_format!`                             |                       | X                             |

## Building locally

The recommended way to build locally is to use the `build.ps1` script: this will build the crate
using all feature combinations, run tests, check formatting, run clippy and build with `#[cfg(no_global_oom_handling)]`
enabled.

In order to run this script you'll need:
* [PowerShell 7+](https://learn.microsoft.com/en-us/powershell/scripting/install/installing-powershell)
* [Rust](https://rustup.rs/)
  * Including the [`rust-src` component](https://rust-lang.github.io/rustup/concepts/components.html).

## Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft 
trademarks or logos is subject to and must follow 
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
