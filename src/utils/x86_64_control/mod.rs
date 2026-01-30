use crate::utils::x86_64_control::{
    cr0::{WRITE_PROTECT, read_cr0, write_cr0},
    msr::{IA32_EFER, rdmsr, wrmsr},
};

pub mod cr0;
pub mod cr2;
pub mod cr3;
pub mod msr;
pub mod segmentation;
pub mod tlb;

pub fn enable_nxe_bit() {
    let nxe_bit = 1 << 11;
    let efer = rdmsr(IA32_EFER);
    wrmsr(IA32_EFER, efer | nxe_bit);
}

pub fn enable_write_protect_bit() {
    let value = read_cr0();
    write_cr0(value | WRITE_PROTECT);
}
