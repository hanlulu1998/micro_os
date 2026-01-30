#![allow(dead_code)]

#[cfg(feature = "use_test")]
pub mod test_frameworks;

pub mod x86_64_control;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use crate::io_port::Port;
    let mut port = Port::<u32>::new(0xf4);
    port.write(exit_code as u32);
}

pub fn get_type<T>(_: T) -> &'static str {
    core::any::type_name::<T>()
}

pub fn align_down(value: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        value & !(align - 1)
    } else if align == 0 {
        value
    } else {
        panic!("Alignment must be a power of two and non-zero");
    }
}

pub fn align_up(value: usize, align: usize) -> usize {
    align_down(value + align - 1, align)
}
