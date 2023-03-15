use core::alloc::Layout;

#[allow(dead_code)]
#[cfg(any(test, not(feature = "use_unstable_apis")))]
mod internal {
    // Forked from the Rust Standard Library: library/alloc/src/collections/mod.rs
    use super::*;

    /// The error type for `try_reserve` methods.
    pub struct TryReserveError {
        pub kind: TryReserveErrorKind,
    }

    /// Details of the allocation that caused a `TryReserveError`
    pub enum TryReserveErrorKind {
        /// Error due to the computed capacity exceeding the collection's maximum
        /// (usually `isize::MAX` bytes).
        CapacityOverflow,

        /// The memory allocator returned an error
        AllocError {
            /// The layout of allocation request that failed
            layout: Layout,
            non_exhaustive: (),
        },
    }

    pub fn build_error_from_layout(layout: Layout) -> alloc::collections::TryReserveError {
        static_assertions::assert_eq_size!(
            alloc::collections::TryReserveError,
            internal::TryReserveError
        );
        unsafe {
            core::mem::transmute(internal::TryReserveError {
                kind: internal::TryReserveErrorKind::AllocError {
                    layout,
                    non_exhaustive: (),
                },
            })
        }
    }
}

#[cfg(feature = "use_unstable_apis")]
fn build_error_from_layout(layout: Layout) -> alloc::collections::TryReserveError {
    alloc::collections::TryReserveErrorKind::AllocError {
        layout,
        non_exhaustive: (),
    }
    .into()
}

#[doc(hidden)]
pub fn alloc_error(layout: Layout) -> alloc::collections::TryReserveError {
    #[cfg(feature = "use_unstable_apis")]
    {
        build_error_from_layout(layout)
    }
    #[cfg(not(feature = "use_unstable_apis"))]
    {
        internal::build_error_from_layout(layout)
    }
}

#[test]
#[cfg(feature = "use_unstable_apis")]
fn check_error_transmute() {
    let layout = core::alloc::Layout::new::<[i32; 42]>();
    assert_eq!(
        build_error_from_layout(layout),
        internal::build_error_from_layout(layout)
    );
}
