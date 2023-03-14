//! Fallible allocation functions for the Rust standard library's [`alloc::vec::Vec`]
//! type.
//!
//! These functions are designed to be usable with `#![no_std]`,
//! `#[cfg(no_global_oom_handling)]`(see <https://github.com/rust-lang/rust/pull/84266>)
//! enabled and Allocators (see <https://github.com/rust-lang/wg-allocators>).
//!
//! # Usage
//!
//! The recommended way to add these functions to `Vec` is by adding a `use`
//! declaration for the `FallibleVec` trait: `use fallible_vec::FallibleVec`:
//! ```
//! # #![feature(allocator_api)]
//! # #[macro_use] extern crate fallible_vec;
//! use fallible_vec::{FallibleVec, try_vec};
//!
//! let mut vec = try_vec![1, 2]?;
//! vec.try_push(3)?;
//! assert_eq!(vec, [1, 2, 3]);
//! # Ok::<(), std::collections::TryReserveError>(())
//! ```
//!
//! # Panic safety
//!
//! These methods are "panic safe", meaning that if a call to external code (e.g.,
//! an iterator's `next()` method or an implementation of `Clone::clone()`)
//! panics, then these methods will leave the `Vec` in a consistent state:
//! * `len()` will be less than or equal to `capacity()`.
//! * Items in `0..len()` will only be items originally in the `Vec` or items
//!   being added to the `Vec`. It will never include uninitialized memory,
//!   duplicated items or dropped items.
//! * Items originally (but no longer) in the `Vec` or being added to (but not
//!   yet in) the `Vec` may be leaked.
//!
//! The exact behavior of each method is specified in its documentations.
//!
//! # Completeness
//!
//! NOTE: This API is incomplete, there are many more infallible functions on
//! `Vec` which have not been ported yet.

#![cfg_attr(not(any(test, doc)), no_std)]
#![feature(allocator_api)]
#![feature(slice_range)]
#![feature(try_reserve_kind)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;
mod collect;
mod set_len_on_drop;

use alloc::{
    collections::{TryReserveError, TryReserveErrorKind},
    vec::Vec,
};
use core::{
    alloc::Allocator,
    ops::{Range, RangeBounds},
    slice,
};
use set_len_on_drop::SetLenOnDrop;

pub use collect::TryCollect;

// These are defined so that the try_vec! and try_vec_in! macros can refer to
// these types in a consistent way: even if the consuming crate doesn't use
// `no_std` and `extern crate alloc`.
#[doc(hidden)]
pub mod alloc_usings {
    pub use alloc::{alloc::Layout, boxed::Box, collections::TryReserveError, vec::Vec};
}

/// Fallible allocation methods for [`Vec`].
pub trait FallibleVec<T, A: Allocator>: Sized {
    /// Extends the `Vec` using the items from the given iterator.
    ///
    /// # Panic safety
    ///
    /// If a call to `next()` on `iter` panics, then all of the items previously
    /// returned from the iterator will be added to the `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    ///
    /// let mut vec = try_vec![1, 2]?;
    /// vec.try_extend([3, 4, 5])?;
    /// assert_eq!(vec, [1, 2, 3, 4, 5]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<(), TryReserveError>;

    /// Appends an element to the back of a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    /// let mut vec = try_vec![1, 2]?;
    /// vec.try_push(3)?;
    /// assert_eq!(vec, [1, 2, 3]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_push(&mut self, item: T) -> Result<(), TryReserveError>;

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    ///
    /// let mut vec = try_vec![1, 2, 3]?;
    /// vec.try_insert(1, 4)?;
    /// assert_eq!(vec, [1, 4, 2, 3]);
    /// vec.try_insert(4, 5)?;
    /// assert_eq!(vec, [1, 4, 2, 3, 5]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_insert(&mut self, index: usize, element: T) -> Result<(), TryReserveError>;

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with the result of
    /// calling the closure `f`. The return values from `f` will end up
    /// in the `Vec` in the order they have been generated.
    ///
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// This method uses a closure to create new values on every push. If
    /// you'd rather [`Clone`] a given value, use [`try_resize`](FallibleVec::try_resize).
    /// If you want to use the [`Default`] trait to generate values, you can
    /// pass [`Default::default`] as the second argument.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    ///
    /// let mut vec = try_vec![1, 2, 3]?;
    /// vec.try_resize_with(5, Default::default)?;
    /// assert_eq!(vec, [1, 2, 3, 0, 0]);
    ///
    /// let mut vec = vec![];
    /// let mut p = 1;
    /// vec.try_resize_with(4, || { p *= 2; p })?;
    /// assert_eq!(vec, [2, 4, 8, 16]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_resize_with<F: FnMut() -> T>(
        &mut self,
        new_len: usize,
        f: F,
    ) -> Result<(), TryReserveError>;

    /// Removes the items in `range` and replaces them with `replace_with` using
    /// the provided allocator for temporary allocations.
    ///
    /// # Panic safety
    ///
    /// If `replace_with` panics on a call to `next()` then the items that were
    /// previously returned by that iterator will either be added to the `Vec`
    /// or dropped. Some of the items after the splicing point (i.e., the end of
    /// `range`) in the `Vec` may be leaked.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    /// use std::alloc::System;
    ///
    /// let mut v = try_vec_in![1, 2, 3, 4 => System]?;
    /// let new = [7, 8, 9];
    /// v.try_splice_in(1..3, new, System)?;
    /// assert_eq!(&v, &[1, 7, 8, 9, 4]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_splice_in<I: IntoIterator<Item = T>>(
        &mut self,
        range: impl RangeBounds<usize>,
        replace_with: I,
        alloc: A,
    ) -> Result<(), TryReserveError>;

    /// Clones and appends all elements in a slice to the `Vec`.
    ///
    /// Iterates over `slice`, clones each element, and then appends
    /// it to this `Vec`. `slice` is traversed in-order.
    ///
    /// Note that this function is same as [`try_extend`]
    /// except that it is specialized to work with slices instead. If and when
    /// Rust gets specialization this function will likely be deprecated (but
    /// still available).
    ///
    /// # Panic safety
    ///
    /// If a call to `clone` for one of the items in `slice` panics, then all
    /// items before the panicking item will have been added to the `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate alloc;
    /// use fallible_vec::*;
    ///
    /// let mut vec = try_vec![1]?;
    /// vec.try_extend_from_slice(&[2, 3, 4])?;
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    ///
    /// [`try_extend`]: Vec::try_extend
    fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), TryReserveError>
    where
        T: Clone;

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `item`.
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// This method will clone the passed value.
    /// If you need more flexibility (or want to rely on [`Default`] instead of
    /// [`Clone`]), use [`try_resize_with`](FallibleVec::try_resize_with).
    /// If you only need to resize to a smaller size, use [`Vec::truncate`].
    ///
    /// # Panic safety
    ///
    /// If a call to `clone` for `item` panics, then the `Vec` will be partially
    /// resized with all of the items cloned before the panic.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate alloc;
    /// use fallible_vec::*;
    ///
    /// let mut vec = try_vec!["hello"]?;
    /// vec.try_resize(3, "world")?;
    /// assert_eq!(vec, ["hello", "world", "world"]);
    ///
    /// let mut vec = try_vec![1, 2, 3, 4]?;
    /// vec.try_resize(2, 0)?;
    /// assert_eq!(vec, [1, 2]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_resize(&mut self, new_len: usize, item: T) -> Result<(), TryReserveError>
    where
        T: Clone;
}

impl<T, A: Allocator> FallibleVec<T, A> for Vec<T, A> {
    fn try_extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<(), TryReserveError> {
        let iter = iter.into_iter();
        let (low_bound, _upper_bound) = iter.size_hint();
        self.try_reserve(low_bound)?;
        for item in iter {
            self.try_push(item)?;
        }
        Ok(())
    }

    fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), TryReserveError>
    where
        T: Clone,
    {
        self.try_reserve(slice.len())?;
        let ptr = self.as_mut_ptr();
        let mut local_len = SetLenOnDrop::new(self);
        for item in slice.iter() {
            unsafe {
                ptr.add(local_len.current_len()).write(item.clone());
            }
            local_len.increment_len(1);
        }

        Ok(())
    }

    fn try_push(&mut self, item: T) -> Result<(), TryReserveError> {
        self.try_reserve(1)?;
        unsafe {
            self.as_mut_ptr().add(self.len()).write(item);
            self.set_len(self.len() + 1);
        }
        Ok(())
    }

    fn try_insert(&mut self, index: usize, element: T) -> Result<(), TryReserveError> {
        move_tail(self, index, 1)?;
        unsafe {
            self.as_mut_ptr().add(index).write(element);
            self.set_len(self.len() + 1);
        }
        Ok(())
    }

    fn try_resize(&mut self, new_len: usize, item: T) -> Result<(), TryReserveError>
    where
        T: Clone,
    {
        #[allow(clippy::comparison_chain)]
        if new_len < self.len() {
            self.truncate(new_len);
        } else if new_len > self.len() {
            self.try_reserve(new_len - self.len())?;
            let ptr = self.as_mut_ptr();
            let mut local_len = SetLenOnDrop::new(self);
            loop {
                unsafe {
                    ptr.add(local_len.current_len()).write(item.clone());
                }
                local_len.increment_len(1);
                if local_len.current_len() == new_len {
                    break;
                }
            }
        }
        Ok(())
    }

    fn try_resize_with<F: FnMut() -> T>(
        &mut self,
        new_len: usize,
        mut f: F,
    ) -> Result<(), TryReserveError> {
        #[allow(clippy::comparison_chain)]
        if new_len < self.len() {
            self.truncate(new_len);
        } else if new_len > self.len() {
            self.try_reserve(new_len - self.len())?;
            let ptr = self.as_mut_ptr();
            let mut local_len = SetLenOnDrop::new(self);
            loop {
                let item = f();
                // Immediately set the length, to protect against panics that occur when calling 'f'.
                unsafe {
                    ptr.add(local_len.current_len()).write(item);
                }
                local_len.increment_len(1);
                if local_len.current_len() == new_len {
                    break;
                }
            }
        }
        Ok(())
    }

    fn try_splice_in<I: IntoIterator<Item = T>>(
        &mut self,
        range: impl RangeBounds<usize>,
        replace_with: I,
        alloc: A,
    ) -> Result<(), TryReserveError> {
        let mut replace_with = replace_with.into_iter();
        let Range {
            start: mut index,
            end,
        } = slice::range(range, ..self.len());

        // Write over the items that need to be removed first.
        while index < end {
            if let Some(item) = replace_with.next() {
                self[index] = item;
                index += 1;
            } else {
                // Nothing else to insert, drop the rest.
                self.drain(index..end);
                return Ok(());
            }
        }

        // If we know roughly how many more there are, copy those directly.
        let (lower_bound, ..) = replace_with.size_hint();
        if lower_bound > 0 {
            move_tail(self, index, lower_bound)?;

            // Temporarily reduce the length: this will result in both the
            // uninitialized memory AND the post-splice items being leaked if a
            // call to next() panics.
            let after_splice = self.len() - index;
            unsafe {
                self.set_len(index);
            }

            {
                let ptr = self.as_mut_ptr();
                let mut local_len = SetLenOnDrop::new(self);
                loop {
                    unsafe {
                        ptr.add(local_len.current_len())
                            .write(replace_with.next().unwrap());
                    }
                    local_len.increment_len(1);
                    if local_len.current_len() == index + lower_bound {
                        break;
                    }
                }
            }

            // Update the index to insert at.
            index += lower_bound;
            // Fixup length to include the port-splice items.
            unsafe {
                self.set_len(self.len() + after_splice);
            }
        }

        // Gather up the remainder and copy those as well.
        let remainder = replace_with.try_collect_in(alloc)?;
        if !remainder.is_empty() {
            move_tail(self, index, remainder.len())?;
            // Don't need to use `SetLenOnDrop` here since we're enumerating
            // over a Vec that we own.
            unsafe {
                self.set_len(self.len() + remainder.len());
            }
            let ptr = unsafe { self.as_mut_ptr().add(index) };
            for (i, item) in remainder.into_iter().enumerate() {
                unsafe { ptr.add(i).write(item) };
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn alloc_error(layout: alloc::alloc::Layout) -> TryReserveError {
    TryReserveErrorKind::AllocError {
        layout,
        non_exhaustive: (),
    }
    .into()
}

/// Creates a [`Vec`] containing the arguments.
///
/// `try_vec!` allows `Vec`s to be defined with the same syntax as array expressions.
/// There are two forms of this macro:
///
/// - Create a [`Vec`] containing a given list of elements:
///
/// ```
/// #![feature(allocator_api)]
/// # #[macro_use] extern crate fallible_vec;
/// let v = try_vec![1, 2, 3]?;
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
///
/// - Create a [`Vec`] from a given element and size:
///
/// ```
/// #![feature(allocator_api)]
/// # #[macro_use] extern crate fallible_vec;
/// let v = try_vec![1; 3]?;
/// assert_eq!(v, [1, 1, 1]);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
///
/// Note that unlike array expressions this syntax supports all elements
/// which implement [`Clone`] and the number of elements doesn't have to be
/// a constant.
///
/// This will use `clone` to duplicate an expression, so one should be careful
/// using this with types having a nonstandard `Clone` implementation. For
/// example, `try_vec![Rc::new(1); 5]` will create a vector of five references
/// to the same boxed integer value, not five references pointing to independently
/// boxed integers.
///
/// Also, note that `try_vec![expr; 0]` is allowed, and produces an empty vector.
/// This will still evaluate `expr`, however, and immediately drop the resulting value, so
/// be mindful of side effects.
///
/// [`Vec`]: alloc::vec::Vec
#[macro_export]
macro_rules! try_vec {
    () => (
        core::result::Result::Ok::<Vec<_>, $crate::alloc_usings::TryReserveError>(
            $crate::alloc_usings::Vec::new())
    );
    ($elem:expr; $n:expr) => (
        $crate::try_new_repeat_item($elem, $n)
    );
    ($($x:expr),+ $(,)?) => ({
        let values = [$($x),+];
        let layout = $crate::alloc_usings::Layout::for_value(&values);
        $crate::alloc_usings::Box::try_new(values)
            .map(|b| <[_]>::into_vec(b))
            .map_err::<$crate::alloc_usings::TryReserveError, _>(|_| $crate::alloc_error(layout))
    });
}

/// Creates a [`Vec`] containing the arguments with the provided allocator.
///
/// `try_vec_in!` allows `Vec`s to be defined with the same syntax as array expressions.
/// There are two forms of this macro:
///
/// - Create a [`Vec`] containing a given list of elements:
///
/// ```
/// #![feature(allocator_api)]
/// # #[macro_use] extern crate fallible_vec;
/// use std::alloc::System;
///
/// let v = try_vec_in![1, 2, 3 => System]?;
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
///
/// - Create a [`Vec`] from a given element and size:
///
/// ```
/// #![feature(allocator_api)]
/// # #[macro_use] extern crate fallible_vec;
/// use std::alloc::System;
///
/// let v = try_vec_in![1; 3 => System]?;
/// assert_eq!(v, [1, 1, 1]);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
///
/// Note that unlike array expressions this syntax supports all elements
/// which implement [`Clone`] and the number of elements doesn't have to be
/// a constant.
///
/// This will use `clone` to duplicate an expression, so one should be careful
/// using this with types having a nonstandard `Clone` implementation. For
/// example, `try_ve_in![Rc::new(1); 5 => allocator]` will create a vector of five references
/// to the same boxed integer value, not five references pointing to independently
/// boxed integers.
///
/// Also, note that `try_vec_in![expr; 0 => allocator]` is allowed, and produces an empty vector.
/// This will still evaluate `expr`, however, and immediately drop the resulting value, so
/// be mindful of side effects.
///
/// [`Vec`]: alloc::vec::Vec
#[macro_export]
macro_rules! try_vec_in {
    ($allocator:expr) => (
        core::result::Result::Ok::<Vec<_, _>, $crate::alloc_usings::TryReserveError>(
            $crate::alloc_usings::Vec::new_in($allocator))
    );
    ($elem:expr; $n:expr => $allocator:expr) => (
        $crate::try_new_repeat_item_in($elem, $n, $allocator)
    );
    ($($x:expr),+ $(,)? => $allocator:expr) => ({
        let values = [$($x),+];
        let layout = $crate::alloc_usings::Layout::for_value(&values);
        $crate::alloc_usings::Box::try_new_in(values, $allocator)
            .map(|b| <[_]>::into_vec(b))
            .map_err::<$crate::alloc_usings::TryReserveError, _>(|_| $crate::alloc_error(layout))
    });
}

/// Constructs a new, empty `Vec<T, A>` with the specified capacity with the
/// provided allocator.
///
/// The vector will be able to hold exactly `capacity` elements without
/// reallocating. If `capacity` is 0, the vector will not allocate.
///
/// It is important to note that although the returned vector has the
/// *capacity* specified, the vector will have a zero *length*. For an
/// explanation of the difference between length and capacity, see
/// *[Capacity and reallocation]*.
///
/// [Capacity and reallocation]: #capacity-and-reallocation
///
/// # Examples
///
/// ```
/// # use fallible_vec::*;
/// use std::alloc::System;
///
/// let mut vec = try_with_capacity_in(10, System)?;
///
/// // The vector contains no items, even though it has capacity for more
/// assert_eq!(vec.len(), 0);
/// assert_eq!(vec.capacity(), 10);
///
/// // These are all done without reallocating...
/// for i in 0..10 {
///     vec.try_push(i)?;
/// }
/// assert_eq!(vec.len(), 10);
/// assert_eq!(vec.capacity(), 10);
///
/// // ...but this may make the vector reallocate
/// vec.try_push(11)?;
/// assert_eq!(vec.len(), 11);
/// assert!(vec.capacity() >= 11);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
pub fn try_with_capacity_in<T, A: Allocator>(
    size: usize,
    alloc: A,
) -> Result<Vec<T, A>, TryReserveError> {
    let mut vec: Vec<T, A> = Vec::new_in(alloc);
    vec.try_reserve(size)?;
    Ok(vec)
}

/// Constructs a new, empty `Vec<T>` with the specified capacity.
///
/// The vector will be able to hold exactly `capacity` elements without
/// reallocating. If `capacity` is 0, the vector will not allocate.
///
/// It is important to note that although the returned vector has the
/// *capacity* specified, the vector will have a zero *length*. For an
/// explanation of the difference between length and capacity, see
/// *[Capacity and reallocation]*.
///
/// [Capacity and reallocation]: #capacity-and-reallocation
///
/// # Examples
///
/// ```
/// # extern crate alloc;
/// # use fallible_vec::*;
/// let mut vec = try_with_capacity(10)?;
///
/// // The vector contains no items, even though it has capacity for more
/// assert_eq!(vec.len(), 0);
/// assert_eq!(vec.capacity(), 10);
///
/// // These are all done without reallocating...
/// for i in 0..10 {
///     vec.try_push(i)?;
/// }
/// assert_eq!(vec.len(), 10);
/// assert_eq!(vec.capacity(), 10);
///
/// // ...but this may make the vector reallocate
/// vec.try_push(11)?;
/// assert_eq!(vec.len(), 11);
/// assert!(vec.capacity() >= 11);
/// # Ok::<(), std::collections::TryReserveError>(())
/// ```
pub fn try_with_capacity<T>(size: usize) -> Result<Vec<T>, TryReserveError> {
    let mut vec: Vec<T> = Vec::new();
    vec.try_reserve(size)?;
    Ok(vec)
}

#[doc(hidden)]
pub fn try_new_repeat_item_in<T: Clone, A: Allocator>(
    item: T,
    size: usize,
    alloc: A,
) -> Result<Vec<T, A>, TryReserveError> {
    try_new_repeat_item_internal(Vec::new_in(alloc), item, size)
}

#[doc(hidden)]
pub fn try_new_repeat_item<T: Clone>(item: T, size: usize) -> Result<Vec<T>, TryReserveError> {
    try_new_repeat_item_internal(Vec::new(), item, size)
}

#[inline]
fn try_new_repeat_item_internal<T: Clone, A: Allocator>(
    mut vec: Vec<T, A>,
    item: T,
    size: usize,
) -> Result<Vec<T, A>, TryReserveError> {
    if size > 0 {
        vec.try_reserve(size)?;
        let ptr = vec.as_mut_ptr();
        let mut local_len = SetLenOnDrop::new(&mut vec);
        loop {
            unsafe {
                ptr.add(local_len.current_len()).write(item.clone());
            }
            local_len.increment_len(1);
            if local_len.current_len() == size {
                break;
            }
        }
    }
    Ok(vec)
}

/// Resizes the `vec` to fit additional elements by moving all of the elements
/// at and after `index` by `by` slots.
///
/// NOTE: Does NOT change the `len` of the `vec`.
fn move_tail<T, A: Allocator>(
    vec: &mut Vec<T, A>,
    index: usize,
    by: usize,
) -> Result<(), TryReserveError> {
    vec.try_reserve(by)?;
    let source = unsafe { vec.as_ptr().add(index) };
    let destination = unsafe { vec.as_mut_ptr().add(index + by) };
    unsafe {
        core::ptr::copy(source, destination, vec.len() - index);
    }
    Ok(())
}

#[cfg(test)]
pub mod tests;
