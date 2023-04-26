// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::FallibleVec;
use crate::TryReserveError;
use alloc::vec::Vec;

#[cfg(feature = "allocator_api")]
use core::alloc::Allocator;

/// Fallible allocations equivalents for [`Iterator::collect`].
pub trait TryCollect<T> {
    /// Attempts to collect items from an iterator into a vector with the provided
    /// allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    /// use std::alloc::System;
    ///
    /// let doubled = [1, 2, 3, 4, 5].map(|i| i * 2);
    /// let vec = doubled.try_collect_in(System)?;
    /// assert_eq!(vec, [2, 4, 6, 8, 10]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    #[cfg(feature = "allocator_api")]
    fn try_collect_in<A: Allocator>(self, alloc: A) -> Result<Vec<T, A>, TryReserveError>;

    /// Attempts to collect items from an iterator into a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #[macro_use] extern crate fallible_vec;
    /// use fallible_vec::*;
    ///
    /// let doubled = [1, 2, 3, 4, 5].map(|i| i * 2);
    /// let vec = doubled.try_collect()?;
    /// assert_eq!(vec, [2, 4, 6, 8, 10]);
    /// # Ok::<(), std::collections::TryReserveError>(())
    /// ```
    fn try_collect(self) -> Result<Vec<T>, TryReserveError>;
}

impl<T, I> TryCollect<T> for I
where
    I: IntoIterator<Item = T>,
{
    #[cfg(feature = "allocator_api")]
    fn try_collect_in<A: Allocator>(self, alloc: A) -> Result<Vec<T, A>, TryReserveError> {
        let mut vec = Vec::new_in(alloc);
        vec.try_extend(self.into_iter())?;
        Ok(vec)
    }

    fn try_collect(self) -> Result<Vec<T>, TryReserveError> {
        let mut vec = Vec::new();
        vec.try_extend(self.into_iter())?;
        Ok(vec)
    }
}
