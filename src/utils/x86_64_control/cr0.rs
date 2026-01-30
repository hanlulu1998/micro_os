use core::arch::asm;

pub const PROTECTED_MODE_ENABLE: u64 = 1;
pub const MONITOR_COPROCESSOR: u64 = 1 << 1;
pub const EMULATE_COPROCESSOR: u64 = 1 << 2;
pub const TASK_SWITCHED: u64 = 1 << 3;
pub const EXTENSION_TYPE: u64 = 1 << 4;
pub const NUMERIC_ERROR: u64 = 1 << 5;
pub const WRITE_PROTECT: u64 = 1 << 16;
pub const ALIGNMENT_MASK: u64 = 1 << 18;
pub const NOT_WRITE_THROUGH: u64 = 1 << 29;
pub const CACHE_DISABLE: u64 = 1 << 30;
pub const PAGING: u64 = 1 << 31;

#[inline]
pub fn read_cr0() -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "mov {}, cr0",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

#[inline]
pub fn write_cr0(val: u64) {
    unsafe {
        asm!(
            "mov cr0, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}
