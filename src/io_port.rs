#![allow(dead_code)]

use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port<T> {
    port: u16,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl Port<u8> {
    pub fn write(&mut self, value: u8) {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") self.port,
                in("al") value,
            );
        }
    }
    pub fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!(
                "in al, dx",
                in("dx") self.port,
                out("al") value,
            );
        }
        value
    }
}

impl Port<u16> {
    pub fn write(&mut self, value: u16) {
        unsafe {
            asm!(
                "out dx, ax",
                in("dx") self.port,
                in("ax") value,
            );
        }
    }

    pub fn read(&self) -> u16 {
        let value: u16;

        unsafe {
            asm!(
                "in ax, dx",
                in("dx") self.port,
                out("ax") value,
            );
        }

        value
    }
}

impl Port<u32> {
    pub fn write(&mut self, value: u32) {
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") self.port,
                in("eax") value,
            );
        }
    }

    pub fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!(
                "in eax, dx",
                in("dx") self.port,
                out("eax") value,
            );
        }
        value
    }
}

impl Port<u64> {
    pub fn write(&mut self, value: u64) {
        unsafe {
            asm!(
                "out dx, rax",
                in("dx") self.port,
                in("rax") value,
            );
        }
    }

    pub fn read(&self) -> u64 {
        let value: u64;
        unsafe {
            asm!(
                "in rax, dx",
                in("dx") self.port,
                out("rax") value,
            );
        }
        value
    }
}
