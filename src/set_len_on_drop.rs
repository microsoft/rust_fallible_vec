// Forked from the Rust Standard Library: library/alloc/src/vec/set_len_on_drop.rs

use alloc::vec::Vec;

pub(super) trait VecType {
    type Item;
}

macro_rules! struct_set_len_on_drop {
    { $(#[doc = $doc:expr])+ struct SetLenOnDrop $impl:tt } => {
        #[cfg(not(feature = "allocator_api"))]
        $(#[doc = $doc])+
        pub(super) struct SetLenOnDrop<'a, T> $impl

        #[cfg(not(feature = "allocator_api"))]
        impl<T> VecType for SetLenOnDrop<'_, T> {
            type Item = Vec<T>;
        }

        #[cfg(feature = "allocator_api")]
        $(#[doc = $doc])+
        pub(super) struct SetLenOnDrop<'a, T, A: core::alloc::Allocator> $impl

        #[cfg(feature = "allocator_api")]
        impl<T, A: core::alloc::Allocator> VecType for SetLenOnDrop<'_, T, A> {
            type Item = Vec<T, A>;
        }
    }
}

struct_set_len_on_drop! {
    /// Set the length of the vec when the `SetLenOnDrop` value goes out of
    /// scope.
    ///
    /// The idea is: The length field in SetLenOnDrop is a local variable
    /// that the optimizer will see does not alias with any stores through the
    /// Vec's data pointer. This is a workaround for alias analysis issue #32155
    struct SetLenOnDrop {
        // NOTE: Using <Self as VecType>::Item doesn't work here since is causes
        // the compiler to insist that `T` and `A` need to have a lifetime of at
        // least `'a`.
        #[cfg(not(feature = "allocator_api"))]
        vec: &'a mut Vec<T>,
        #[cfg(feature = "allocator_api")]
        vec: &'a mut Vec<T, A>,
        local_len: usize,
    }
}

macro_rules! impl_set_len_on_drop {
    { impl SetLenOnDrop $impl:tt } => {
        #[cfg(not(feature = "allocator_api"))]
        impl<'a, T> SetLenOnDrop<'a, T> $impl

        #[cfg(feature = "allocator_api")]
        impl<'a, T, A: core::alloc::Allocator> SetLenOnDrop<'a, T, A> $impl
    };
    { impl $trait:ident for SetLenOnDrop $impl:tt } => {
        #[cfg(not(feature = "allocator_api"))]
        impl<T> $trait for SetLenOnDrop<'_, T> $impl

        #[cfg(feature = "allocator_api")]
        impl<T, A: core::alloc::Allocator> $trait for SetLenOnDrop<'_, T, A> $impl
    }
}

impl_set_len_on_drop! {
    impl SetLenOnDrop {
        #[inline]
        pub(super) fn new(vec: &'a mut <Self as VecType>::Item) -> Self {
            SetLenOnDrop {
                local_len: vec.len(),
                vec,
            }
        }

        #[inline]
        pub(super) fn increment_len(&mut self, increment: usize) {
            self.local_len += increment;
        }

        #[inline]
        pub(super) fn current_len(&self) -> usize {
            self.local_len
        }
    }
}

impl_set_len_on_drop! {
    impl Drop for SetLenOnDrop {
        #[inline]
        fn drop(&mut self) {
            unsafe {
                self.vec.set_len(self.local_len);
            }
        }
    }
}
