use core::arch::asm;

#[inline]
pub fn read_cr2() -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

#[inline]
pub fn write_cr2(val: u64) {
    unsafe {
        asm!(
            "mov cr2, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}