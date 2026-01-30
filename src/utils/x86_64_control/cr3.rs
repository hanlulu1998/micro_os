use core::arch::asm;

#[inline]
pub fn read_cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

#[inline]
pub fn write_cr3(val: u64) {
    unsafe {
        asm!(
            "mov cr3, {}",
            in(reg) val,
            options(nostack, preserves_flags)
        );
    }
}
