// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::*;
use alloc::{alloc::Global, vec::Vec};
use core::sync::atomic::{AtomicI32, Ordering};
use std::{alloc::System, cell::Cell};

#[derive(Default)]
struct ExplodingCloner<'a> {
    clone_panics: Cell<bool>,
    drop_counter: Option<&'a AtomicI32>,
}

impl Clone for ExplodingCloner<'_> {
    fn clone(&self) -> Self {
        if self.clone_panics.replace(true) {
            panic!("BOOM");
        }
        Self {
            clone_panics: Default::default(),
            drop_counter: self.drop_counter.clone(),
        }
    }
}

impl Drop for ExplodingCloner<'_> {
    fn drop(&mut self) {
        if let Some(drop_counter) = &self.drop_counter {
            drop_counter.fetch_add(1, Ordering::Relaxed);
        }
    }
}

struct ExplodingIterator {
    value: i32,
    panic_at: i32,
    lower_bound_hint: usize,
}

impl Iterator for ExplodingIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.value += 1;
        if self.value == self.panic_at {
            panic!("BOOM");
        }

        Some(self.value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.lower_bound_hint - self.value as usize), None)
    }
}

#[test]
fn test_push() {
    let mut v = Vec::new();
    v.try_push(1).unwrap();
    assert_eq!(v, [1]);
    v.try_push(2).unwrap();
    assert_eq!(v, [1, 2]);
    v.try_push(3).unwrap();
    assert_eq!(v, [1, 2, 3]);
}

#[test]
fn test_extend_from_slice() {
    let a: Vec<isize> = try_vec![1, 2, 3, 4, 5].unwrap();
    let b: Vec<isize> = try_vec![6, 7, 8, 9, 0].unwrap();

    let mut v: Vec<isize> = a;

    v.try_extend_from_slice(&b).unwrap();

    assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
}

#[test]
fn test_splice() {
    let mut v = try_vec![1, 2, 3, 4, 5].unwrap();
    let a = [10, 11, 12];
    v.try_splice_in(2..4, a, Global).unwrap();
    assert_eq!(v, &[1, 2, 10, 11, 12, 5]);
    v.try_splice_in(1..3, Some(20), Global).unwrap();
    assert_eq!(v, &[1, 20, 11, 12, 5]);
}

#[test]
fn test_splice_inclusive_range() {
    let mut v = try_vec![1, 2, 3, 4, 5].unwrap();
    let a = [10, 11, 12];
    v.try_splice_in(2..=3, a, Global).unwrap();
    assert_eq!(v, &[1, 2, 10, 11, 12, 5]);
    v.try_splice_in(1..=2, Some(20), Global).unwrap();
    assert_eq!(v, &[1, 20, 11, 12, 5]);
}

#[test]
#[should_panic]
fn test_splice_out_of_bounds() {
    let mut v = try_vec![1, 2, 3, 4, 5].unwrap();
    let a = [10, 11, 12];
    v.try_splice_in(5..6, a, Global).unwrap();
}

#[test]
#[should_panic]
fn test_splice_inclusive_out_of_bounds() {
    let mut v = try_vec![1, 2, 3, 4, 5].unwrap();
    let a = [10, 11, 12];
    v.try_splice_in(5..=5, a, Global).unwrap();
}

#[test]
fn test_splice_items_zero_sized() {
    let mut vec = try_vec![(); 3].unwrap();
    let vec2 = try_vec![].unwrap();
    vec.try_splice_in(1..2, vec2.iter().cloned(), Global)
        .unwrap();
    assert_eq!(vec, &[(), ()]);
}

#[test]
fn test_splice_unbounded() {
    let mut vec = try_vec![1, 2, 3, 4, 5].unwrap();
    vec.try_splice_in(.., None, Global).unwrap();
    assert_eq!(vec, &[]);
}

#[test]
fn test_into_boxed_slice() {
    let xs = try_vec![1, 2, 3].unwrap();
    let ys = xs.into_boxed_slice();
    assert_eq!(&*ys, [1, 2, 3]);
}

// regression test for issue #85322. Peekable previously implemented InPlaceIterable,
// but due to an interaction with IntoIter's current Clone implementation it failed to uphold
// the contract.
#[test]
fn test_collect_after_iterator_clone() {
    let v = try_vec_in![0; 5 => Global].unwrap();
    let mut i = v.into_iter().map(|i| i + 1).peekable();
    i.peek();
    let v = i.clone().try_collect().unwrap();
    assert_eq!(v, [1, 1, 1, 1, 1]);
    assert!(v.len() <= v.capacity());
}

#[test]
fn test_macro_forms() {
    let v: Vec<i32> = try_vec![].unwrap();
    assert_eq!(v, vec![]);
    assert_eq!(try_vec!['c'; 10].unwrap(), vec!['c'; 10]);
    assert_eq!(try_vec![1, 2, 3, 4].unwrap(), vec![1, 2, 3, 4]);

    let v: Vec<i32> = try_vec_in![Global].unwrap();
    assert_eq!(v, vec![]);
    assert_eq!(try_vec_in!['c'; 10 => Global].unwrap(), vec!['c'; 10]);
    assert_eq!(try_vec_in![1, 2, 3, 4 => Global].unwrap(), vec![1, 2, 3, 4]);

    // Explicit typing to ensure that the allocator is passed through.
    let _v: Vec<i32, System> = try_vec_in![System].unwrap();
    let _v: Vec<char, System> = try_vec_in!['c'; 10 => System].unwrap();
    let _v: Vec<i32, System> = try_vec_in![1, 2, 3, 4 => System].unwrap();
}
#[test]
fn test_zst() {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct ZeroSized {}

    // Verify macros
    let mut v: Vec<ZeroSized> = try_vec![].unwrap();
    assert_eq!(v, vec![]);
    assert_eq!(try_vec![ZeroSized{}; 10].unwrap(), vec![ZeroSized {}; 10]);
    assert_eq!(
        try_vec![ZeroSized {}, ZeroSized {}].unwrap(),
        vec![ZeroSized {}, ZeroSized {}]
    );

    // Verify functions.
    v.try_push(ZeroSized {}).unwrap();
    assert_eq!(v.len(), 1);
    v.try_resize(42, ZeroSized {}).unwrap();
    assert_eq!(v.len(), 42);
}

#[test]
fn test_panic_during_resize_with() {
    let mut v = try_vec![].unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut will_panic = false;
            v.try_resize_with(42, || {
                if will_panic {
                    panic!()
                } else {
                    will_panic = true;
                    "Hello"
                }
            })
            .unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    assert_eq!(v, &["Hello"]);
}

#[test]
fn test_panic_during_resize() {
    let mut v = try_vec![].unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            v.try_resize(42, ExplodingCloner::default()).unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    assert_eq!(v.len(), 1);
}

#[test]
fn test_panic_during_extend_from_slice() {
    let mut v = try_vec![].unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let clone_from = [
                ExplodingCloner::default(),
                ExplodingCloner {
                    clone_panics: Cell::new(true),
                    drop_counter: None,
                },
                ExplodingCloner::default(),
            ];
            v.try_extend_from_slice(&clone_from).unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    assert_eq!(v.len(), 1);
}

#[test]
fn test_panic_during_splice_in_before_lower_bound() {
    let mut v = try_vec![10, 20, 30, 40].unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            v.try_splice_in(
                1..3,
                ExplodingIterator {
                    value: 0,
                    panic_at: 10,
                    lower_bound_hint: 100,
                },
                Global,
            )
            .unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    // Resulted in partial splice: items before panic are inserted.
    assert_eq!(v, &[10, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_panic_during_splice_in_after_lower_bound() {
    let mut v = try_vec![10, 20, 30, 40].unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            v.try_splice_in(
                1..3,
                ExplodingIterator {
                    value: 0,
                    panic_at: 100,
                    lower_bound_hint: 5,
                },
                Global,
            )
            .unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    // Resulted in partial splice: items before lower bound hint are inserted.
    assert_eq!(v, &[10, 1, 2, 3, 4, 5, 40]);
}

#[test]
fn test_panic_during_try_vec_runs_drop() {
    let drop_counter = AtomicI32::new(0);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let item = ExplodingCloner {
                clone_panics: Default::default(),
                drop_counter: Some(&drop_counter),
            };
            let _ = try_vec![item; 42].unwrap();
        }))
        .is_err(),
        "Panic was not propagated"
    );

    // Should have dropped the original ExplodingCloner AND the one that was inserted.
    assert_eq!(drop_counter.load(Ordering::Relaxed), 2);
}
