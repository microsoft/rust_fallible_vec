// Forked from the Rust Standard Library: rust/library/alloc/src/vec/set_len_on_drop.rs

use alloc::vec::Vec;
use core::alloc::Allocator;

/// Set the length of the vec when the `SetLenOnDrop` value goes out of scope.
///
/// The idea is: The length field in SetLenOnDrop is a local variable
/// that the optimizer will see does not alias with any stores through the Vec's data
/// pointer. This is a workaround for alias analysis issue #32155
pub(super) struct SetLenOnDrop<'a, T, A: Allocator> {
    vec: &'a mut Vec<T, A>,
    local_len: usize,
}

impl<'a, T, A: Allocator> SetLenOnDrop<'a, T, A> {
    #[inline]
    pub(super) fn new(vec: &'a mut Vec<T, A>) -> Self {
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

impl<T, A: Allocator> Drop for SetLenOnDrop<'_, T, A> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.vec.set_len(self.local_len);
        }
    }
}
