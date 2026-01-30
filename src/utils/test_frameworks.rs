use crate::{serial_print, serial_println};
use core::panic::PanicInfo;

#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed(&'static str),
}

type TestFn = fn() -> TestResult;

pub struct TestCase {
    pub name: &'static str,
    pub func: TestFn,
}

#[linkme::distributed_slice]
pub static TEST_REGISTRY: [TestCase] = [..];

#[macro_export]
macro_rules! test_case {
    ($func:path) => {
        paste::paste! {
            #[linkme::distributed_slice(TEST_REGISTRY)]
            static [<TEST_CASE_ $func:upper>]: TestCase = TestCase {
            name: stringify!($func),
            func: $func,
        };
        }
    };
}

#[macro_export]
macro_rules! expect_eq {
    ($a:expr, $b:expr, $msg:expr) => {
        if $a != $b {
            return TestResult::Failed($msg);
        }
    };
    ($a:expr, $b:expr) => {
        if $a != $b {
            return TestResult::Failed(concat!(
                "Assertion test failed: ",
                stringify!($a),
                " != ",
                stringify!($b),
            ));
        }
    };
}

#[macro_export]
macro_rules! assert_has_not_been_called {
    () => {
        assert_has_not_been_called!("assertion failed: has_run == false");
    };
    ($($arg:tt)+) => {{
        fn assert_has_not_been_called() {
            use core::sync::atomic::{AtomicBool, Ordering};
            static CALLED: AtomicBool = AtomicBool::new(false);
            let called = CALLED.swap(true, Ordering::Relaxed);
            assert!(called == false, $($arg)+);
        }
        assert_has_not_been_called();
    }};
}

use super::{QemuExitCode, exit_qemu};

pub fn test_main() {
    let mut passed = 0;
    let mut failed = 0;

    serial_println!("Running {} tests", TEST_REGISTRY.len());

    for test in TEST_REGISTRY.iter() {
        serial_print!("{}\t", test.name);
        match (test.func)() {
            TestResult::Passed => {
                serial_println!("[PASSED]\t");
                passed += 1;
            }
            TestResult::Failed(msg) => {
                serial_println!("[FAILED]: {}", msg);
                failed += 1;
            }
        }
    }

    serial_println!("Test Summary: {} passed, {} failed", passed, failed);
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("\nError: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
