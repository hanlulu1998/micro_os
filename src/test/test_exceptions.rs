use core::arch::asm;

use crate::utils::test_frameworks::TestResult;

pub fn divide_by_zero() -> TestResult {
    unsafe {
        asm!(
            "mov dx, 0",
            "div dx",
            out("ax") _,
            out("dx") _,
            options(nostack, nomem, preserves_flags, raw)
        );
    }
    TestResult::Passed
}


pub fn invalid_opcode()->TestResult{
    unsafe { asm!("ud2") };
    TestResult::Passed
}


pub fn page_fault()->TestResult{
    unsafe { *(0xdeadbea0 as *mut u64) = 42 };
    TestResult::Passed
}