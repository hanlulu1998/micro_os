use core::arch::asm;

use crate::utils::x86_64_control::cr3::{read_cr3, write_cr3};

#[inline]
pub fn tlb_flush(addr: u64) {
    unsafe {
        asm!(
            "invlpg [{}]",      // 汇编模板，使用输入寄存器中的地址
            in(reg) addr,       // 输入操作数（64位地址）
            options(nostack, preserves_flags) // 不修改栈、保留标志寄存器
        );
    }
}

pub fn tlb_flush_all() {
    let value = read_cr3();
    write_cr3(value);
}
