use core::arch::naked_asm;

use spin::{Lazy, Once};

use crate::{
    memory::MemoryController,
    println,
    utils::x86_64_control::{
        self,
        gdt::{Descriptor, Gdt},
        segmentation::{SegmentSelector, TaskStateSegment, load_tss, set_cs},
    },
};

mod idt;

macro_rules! handler {
    ($name: ident) => {{
        #[unsafe(naked)]
        extern "C" fn wrapper()->!{
            naked_asm!(
                    // 保存scratch registers
                    "push rax
                    push rcx
                    push rdx
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11",


                    "mov rdi, rsp",
                    "add rdi, 9*8",
                    "call {handler}",

                    // 恢复scratch registers
                    "pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rdx
                    pop rcx
                    pop rax",

                    "iretq",
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
                    // 保存scratch registers
                    "push rax
                    push rcx
                    push rdx
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11",

                    "mov rsi, [rsp + 9*8]", // 加载错误码到rsi
                    "mov rdi, rsp",
                    "add rdi, 10*8",    // 计算异常栈帧指针
                    "sub rsp, 8",       // 为对齐栈帧留出空间
                    "call {handler}",
                    "add rsp, 8",

                    // 恢复scratch registers
                    "pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rdx
                    pop rcx
                    pop rax",

                    "add rsp, 8", //弹出错误码
                    "iretq",
                    handler = sym $name
            );
        }
        wrapper
    }}
}

static IDT: Lazy<idt::Idt> = Lazy::new(|| {
    let mut idt = idt::Idt::new();
    idt.set_handler(0, handler!(divide_by_zero_handler));
    idt.set_handler(3, handler!(breakpoint_handler)); // new
    idt.set_handler(6, handler!(invalid_opcode_handler));
    idt.set_handler(8, handler_with_error_code!(double_fault_handler));
    idt.set_stack_index(8, DOUBLE_FAULT_IST_INDEX as u16);
    // idt.set_handler(14, handler_with_error_code!(page_fault_handler));
    idt
});

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<Gdt> = Once::new();

const DOUBLE_FAULT_IST_INDEX: usize = 1;
pub fn init(memory_controller: &mut MemoryController) {
    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("could not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = double_fault_stack.top();
        tss
    });
    let mut code_selector = SegmentSelector::zero();
    let mut tss_selector = SegmentSelector::zero();
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();
    unsafe {
        set_cs(code_selector);
        load_tss(tss_selector);
    }

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

extern "C" fn divide_by_zero_handler(stack_frame: *const ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", unsafe {
        &*stack_frame
    });
    loop {}
}

extern "C" fn invalid_opcode_handler(stack_frame: *const ExceptionStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    println!(
        "\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
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

extern "C" fn page_fault_handler(stack_frame: *const ExceptionStackFrame, error_code: u64) {
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\
        \nerror code: {:?}\n{:#?}",
        x86_64_control::cr2::read_cr2(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        unsafe { &*stack_frame }
    );
}

extern "C" fn breakpoint_handler(stack_frame: *const ExceptionStackFrame) {
    let stack_frame = unsafe { &*stack_frame };
    println!(
        "\nEXCEPTION: BREAKPOINT at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
}

extern "C" fn double_fault_handler(stack_frame: *const ExceptionStackFrame, _error_code: u64) {
    let stack_frame = unsafe { &*stack_frame };
    println!("\nEXCEPTION: DOUBLE FAULT");
    println!("ExceptionStackFrame {{");
    println!(
        "    instruction_pointer: {},",
        stack_frame.instruction_pointer
    );
    println!("    code_segment: {},", stack_frame.code_segment);
    println!("    cpu_flags: {},", stack_frame.cpu_flags);
    println!("    stack_pointer: {},", stack_frame.stack_pointer);
    println!("    stack_segment: {},", stack_frame.stack_segment);
    println!("}}");

    loop {}
}
