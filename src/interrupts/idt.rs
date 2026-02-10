use crate::utils::x86_64_control::segmentation::{
    Dtr, PrivilegeLevel, SegmentSelector, lidt, segment_register,
};

pub type HandlerFunc = extern "C" fn() -> !;

#[derive(Debug)]
pub struct Idt([Entry; 16]);

impl Idt {
    pub fn new() -> Self {
        Idt([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOptions {
        self.0[entry as usize] = Entry::new(segment_register::CS::get_reg_selector(), handler);
        &mut self.0[entry as usize].options
    }

    pub fn set_stack_index(&mut self, entry: u8, index: u16) {
        self.0[entry as usize]
            .options
            .set_ist_bits(index as u8 & 0b111);
    }

    pub fn list(&self, index: usize) -> &Entry {
        &self.0[index]
    }

    pub fn load(&'static self) {
        let ptr = Dtr {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Entry {
            pointer_low: pointer as u16,
            gdt_selector: gdt_selector,
            options: EntryOptions::new(),
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            reserved: 0,
        }
    }

    fn missing() -> Self {
        Entry {
            pointer_low: 0,
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            options: EntryOptions::minimal(),
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }

    pub fn options(&self) -> EntryOptions {
        self.options
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> Self {
        let mut options = 0;
        options |= 0b111 << 9;
        Self(options)
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        if present {
            self.0 |= 1 << 15;
        } else {
            self.0 &= !(1 << 15);
        }
        self
    }

    pub fn set_ist_bits(&mut self, index: u8) {
        // 清零 bit 0–2
        self.0 &= !(0b111 << 0);
        // 设置新值（只取低 3 位）
        self.0 |= ((index as u16) & 0b111) << 0;
    }

    pub fn set_gate_type(&mut self, ty: u8) {
        // 先清零 bit 8–11（4 位）
        self.0 &= !(0b1111 << 8);
        // 再设置新的类型值（只取低 4 位）
        self.0 |= ((ty as u16) & 0b1111) << 8;
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0 &= !(0b11 << 13);
        self.0 |= (dpl & 0b11) << 13;
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        if disable {
            self.0 |= 1 << 8;
        } else {
            self.0 &= !(1 << 8);
        }
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0 &= !0b111;
        self.0 |= index & 0b111;
        self
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }
}
