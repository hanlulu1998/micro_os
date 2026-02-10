#![no_std]
#![allow(dead_code)]
mod interrupts;
mod io_port;
mod memory;
mod multiboot_info;
mod serial;
mod utils;
mod vga_buffer;
extern crate alloc;

#[cfg(feature = "use_test")]
mod test;

use core::panic::PanicInfo;

#[cfg(feature = "use_test")]
use utils::test_frameworks::*;

#[cfg(feature = "use_test")]
use crate::test::{test_allocator::*, test_exceptions::*};

#[unsafe(naked)]
extern "C" fn naked_function_example() {
    core::arch::naked_asm!("mov rax, 0x42", "ret");
}

#[cfg(feature = "use_test")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(multiboot_information_address: usize) -> ! {
    // 自定义测试
    // test_paging(multiboot_information_address);
    // test_remap_the_kernel(multiboot_information_address);

    let boot_info = crate::multiboot_info::MultibootInfo::new(multiboot_information_address);

    let mut memory_controller = memory::init(&boot_info);

    interrupts::init(&mut memory_controller);

    // naked_function_example();

    // test_main();

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    println!("It did not crash!");
    loop {}
}

#[cfg(not(feature = "use_test"))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(multiboot_information_address: usize) -> ! {
    loop {}
}

#[cfg(feature = "use_test")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(not(feature = "use_test"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// #[cfg(feature = "use_test")]
// test_case!(simple_allocation);
// #[cfg(feature = "use_test")]
// test_case!(large_vec);
// #[cfg(feature = "use_test")]
// test_case!(many_boxes);
// #[cfg(feature = "use_test")]
// test_case!(many_boxes_long_lived);

// #[cfg(feature = "use_test")]
// test_case!(divide_by_zero);

// #[cfg(feature = "use_test")]
// test_case!(invalid_opcode);
#[cfg(feature = "use_test")]
test_case!(breakpoint);

#[cfg(feature = "use_test")]
test_case!(page_fault);
