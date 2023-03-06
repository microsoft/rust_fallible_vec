# Fallible allocation functios for Vec

Fallible allocation functions for the Rust standard library's [`alloc::vec::Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) type.

These functions are designed to be usable with `#![no_std]` and `#[cfg(no_global_oom_handling)]`(see <https://github.com/rust-lang/rust/pull/84266>) enabled.

## Usage

The recommended way to add these functions to `Vec` is by adding a `use` declaration for the entire module: `use fallible_vec::*`:
```rust
use fallible_vec::*;
let mut vec = try_vec![1, 2]?;
vec.try_push(3)?;
assert_eq!(vec, [1, 2, 3]);
```

## Panic Safety

These methods are "panic safe", meaning that if a call to external code (e.g., an iterator's `next()` method or an implementation of `Clone::clone()`) panics, then these methods will leave the `Vec` in a consistent state:
* `len()` will be less than or equal to `capacity()`.
* Items in `0..len()` will only be items originally in the `Vec` or items being added to the `Vec`. It will never include uninitialized memory, duplicated items or dropped items.
* Items originally (but no longer) in the `Vec` or being added to (but not yet in) the `Vec` may be leaked - any method that may leak items like this will have a note to specify its behavior.

The exact behavior of each method is specified in its documentations.

## Code origin

Most of this code is forked form [Rust's Standard Library](https://github.com/rust-lang/rust). While we will attempt to keep the code and docs in sync, if you notice any issues please check if they have been fixed in the Standard Library first.

## This API is incomplete

There are many more infallible functions on `Vec` which have not been ported yet. If there's a particular API that you're missing feel free to open a PR or file an Issue to get it added.

## Why are these not already in the Standard Library

There is a [PR to add these and more](https://github.com/rust-lang/rust/pull/95051) to the Standard Library, followed by an [RFC to discuss if it's a good idea or not to do so](https://github.com/rust-lang/rfcs/pull/3271).

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
