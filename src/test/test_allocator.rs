use crate::{expect_eq, memory::allocator::HEAP_SIZE, utils::test_frameworks::TestResult};
use alloc::{boxed::Box, vec::Vec};

pub fn simple_allocation() -> TestResult {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    expect_eq!(*heap_value_1, 41);
    expect_eq!(*heap_value_2, 13);
    TestResult::Passed
}

pub fn large_vec() -> TestResult {
    let n: u64 = 1000;
    let mut vec = Vec::new();

    for i in 0..n {
        vec.push(i);
    }
    expect_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    TestResult::Passed
}

pub fn many_boxes() -> TestResult {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        expect_eq!(*x, i);
    }
    TestResult::Passed
}

pub fn many_boxes_long_lived() -> TestResult {
    let long_lived = Box::new(1); // new
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        expect_eq!(*x, i);
    }
    expect_eq!(*long_lived, 1); // new
    TestResult::Passed
}
