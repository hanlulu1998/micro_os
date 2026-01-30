use core::arch::naked_asm;

use spin::Lazy;

use crate::{println, utils::x86_64_control};

mod idt;


macro_rules! handler {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper()->!{
            naked_asm!("mov rdi, rsp",          // 直接传 rsp
                    "sub rsp, 8",       // 为对齐栈帧留出空间
                    "call {handler}",
                    handler = sym $name
            );
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper()->!{
            naked_asm!(
                    "pop rsi",          // 将错误代码放入 rsi
                    "mov rdi, rsp",          // 直接传 rsp
                    "sub rsp, 8",       // 为对齐栈帧留出空间
                    "call {handler}",
                    handler = sym $name
            );
        }
        wrapper
    }}
}

static IDT: Lazy<idt::Idt> = Lazy::new(|| {
    let mut idt = idt::Idt::new();
    idt.set_handler(0, handler!(divide_by_zero_handler));
    idt.set_handler(6, handler!(invalid_opcode_handler));
    idt.set_handler(14,handler_with_error_code!(page_fault_handler));
    idt
});

pub fn init() {
    IDT.load();
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

extern "C" fn divide_by_zero_handler(stack_frame: *const ExceptionStackFrame) -> ! {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", unsafe {
        &*stack_frame
    });
    loop {}
}

extern "C" fn invalid_opcode_handler(stack_frame: *const ExceptionStackFrame)
    -> !
{
    let stack_frame = unsafe { &*stack_frame };
    println!("\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame);
    loop {}
}

use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}


extern "C" fn page_fault_handler(stack_frame: * const ExceptionStackFrame,
                                 error_code: u64) -> !
{
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\
        \nerror code: {:?}\n{:#?}",
        x86_64_control::cr2::read_cr2(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        unsafe { &*stack_frame });
    loop {}
}