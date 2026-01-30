use core::arch::asm;

pub mod segment_register {

    use crate::utils::x86_64_control::segmentation::SegmentSelector;

    pub struct CS;
    pub struct DS;
    pub struct ES;
    pub struct SS;

    pub struct FS;
    pub struct GS;

    macro_rules! get_selector_template {
    ($name:literal) => {
            pub fn get_reg_selector()->SegmentSelector{
                let value: u16;
                unsafe {
                    core::arch::asm!(
                        concat!("mov {0:x}, ", $name),
                        out(reg) value,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                SegmentSelector(value)
            }
        };
    }

    impl CS {
        get_selector_template!("cs");
    }

    impl SS {
        get_selector_template!("ss");
    }

    impl DS {
        get_selector_template!("ds");
    }

    impl ES {
        get_selector_template!("es");
    }

    impl FS {
        get_selector_template!("fs");
    }

    impl GS {
        get_selector_template!("gs");
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    #[inline]
    pub const fn from_u16(value: u16) -> PrivilegeLevel {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    #[inline]
    pub fn index(self) -> u16 {
        (self.0 >> 3) & 0x1FFF
    }

    #[inline]
    pub fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0 & 0b11_u16)
    }

    #[inline]
    pub fn set_rpl(&mut self, rpl: PrivilegeLevel) {
        self.0 = (self.0 & !0b11_u16) | (rpl as u16);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct Dtr {
    pub limit: u16,
    pub base: u64,
}

#[inline]
pub unsafe fn lidt(idt: &Dtr) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}
