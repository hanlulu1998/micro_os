use crate::utils::align_up;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultibootTagType {
    End = 0,
    MemoryMap = 6,
    ElfSections = 9,
    LoadBaseAddr = 21,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootInfoHeader {
    pub total_size: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootTagHeader {
    pub tag_type: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootElfSymbolsTag {
    pub header: MultibootTagHeader,
    pub num: u16,
    pub entsize: u16,
    pub shndx: u16,
    pub reserved: u16,
    pub section_headers: [Elf64SectionHeader; 0],
}

impl MultibootElfSymbolsTag {
    pub fn entry_num(&self) -> usize {
        (self.header.size as usize - core::mem::size_of::<MultibootElfSymbolsTag>())
            / core::mem::size_of::<Elf64SectionHeader>()
    }

    pub fn sections(&self) -> &[Elf64SectionHeader] {
        let num_entries = self.entry_num();
        unsafe { core::slice::from_raw_parts(self.section_headers.as_ptr(), num_entries) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64SectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,
    _reserved1: u32,
    pub sh_flags: u32,
    _reserved2: u32,
    pub sh_addr: u32,
    _reserved3: u32,
    pub sh_offset: u32,
    _reserved4: u32,
    pub sh_size: u32,
    pub sh_link: u32,
    pub sh_info: u32,
    _reserved5: u32,
    pub sh_addralign: u32,
    _reserved6: u32,
    pub sh_entsize: u32,
}

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SectionFlags: u32 {
    const SHF_WRITE = 0x1;
    const SHF_ALLOC = 0x2;
    const SHF_EXECINSTR = 0x4;
    const SHF_MERGE = 0x10;
    const SHF_STRINGS = 0x20;
    const SHF_INFO_LINK = 0x40;
    const SHF_LINK_ORDER = 0x80;
    const SHF_OS_NONCONFORMING = 0x100;
    const SHF_GROUP = 0x200;
    const SHF_TLS = 0x400;
    const SHF_COMPRESSED = 0x800;
    }
}

impl Elf64SectionHeader {
    pub fn is_allocated(&self) -> bool {
        self.flags().contains(SectionFlags::SHF_ALLOC)
    }

    pub fn start_address(&self) -> usize {
        self.sh_addr as usize
    }

    pub fn size(&self) -> usize {
        self.sh_size as usize
    }

    pub fn end_address(&self) -> usize {
        self.sh_addr as usize + self.sh_size as usize
    }

    pub fn flags(&self) -> SectionFlags {
        SectionFlags::from_bits_truncate(self.sh_flags)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootMemMapTag {
    pub header: MultibootTagHeader,
    pub entry_size: u32,
    pub entry_version: u32,
    pub entries: [MultibootMemMapEntry; 0],
}

impl MultibootMemMapTag {
    pub fn entry_num(&self) -> usize {
        (self.header.size as usize - core::mem::size_of::<MultibootMemMapTag>())
            / self.entry_size as usize
    }

    pub fn entries(&self) -> &[MultibootMemMapEntry] {
        let num_entries = self.entry_num();
        unsafe { core::slice::from_raw_parts(self.entries.as_ptr(), num_entries) }
    }
}

pub enum MemoryMapEntryType {
    Available = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    Nvs = 4,
    BadRam = 5,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootMemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootLoadBaseAddrTag {
    pub header: MultibootTagHeader,
    pub load_base_addr: u32,
}

#[derive(Debug)]
pub struct MultibootInfo {
    base_address: usize,
}

#[derive(Debug)]
pub struct MultibootAddressSection {
    pub kernel_start: usize,
    pub kernel_end: usize,
    pub multiboot_start: usize,
    pub multiboot_end: usize,
}

impl MultibootInfo {
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    pub fn get_boot_info_total_size(&self) -> usize {
        let header = unsafe { &*(self.base_address as *const MultibootInfoHeader) };
        header.total_size as usize
    }

    pub fn start_address(&self) -> usize {
        self.base_address
    }

    pub fn end_address(&self) -> usize {
        self.base_address + self.get_boot_info_total_size()
    }

    pub fn get_boot_info_base_address(&self) -> usize {
        self.base_address
    }

    pub fn get_tag<T: Sized>(&self, tag_type: MultibootTagType) -> Option<&T> {
        let mut tag_base_address = self.base_address + 4 * 2;
        loop {
            let tag = unsafe { &*(tag_base_address as *const MultibootTagHeader) };
            if tag.tag_type == MultibootTagType::End as u32 {
                return None;
            }
            if tag.tag_type == tag_type as u32 {
                return Some(unsafe { &*(tag_base_address as *const T) });
            }
            tag_base_address += align_up(tag.size as usize, 8);
        }
    }

    pub fn get_memory_entries(&self) -> &[MultibootMemMapEntry] {
        let mem_map_tag = self
            .get_tag::<MultibootMemMapTag>(MultibootTagType::MemoryMap)
            .expect("No memory map found");

        mem_map_tag.entries()
    }

    pub fn get_elf_sections(&self) -> &[Elf64SectionHeader] {
        let elf_section_tag = self
            .get_tag::<MultibootElfSymbolsTag>(MultibootTagType::ElfSections)
            .expect("No ELF sections found");

        elf_section_tag.sections()
    }

    pub fn get_multiboot_address_section(&self) -> MultibootAddressSection {
        let elf_section_tag = self
            .get_tag::<MultibootElfSymbolsTag>(MultibootTagType::ElfSections)
            .expect("No ELF sections found");

        let kernel_start = elf_section_tag
            .sections()
            .iter()
            .filter(|s| s.sh_size > 0)
            .map(|s| s.sh_addr as usize)
            .min()
            .expect("No ELF sections found");

        let kernel_end = elf_section_tag
            .sections()
            .iter()
            .map(|s| (s.sh_addr + s.sh_size) as usize)
            .max()
            .expect("No ELF sections found");

        let multiboot_start = self.get_boot_info_base_address();
        let multiboot_end = multiboot_start + self.get_boot_info_total_size();

        MultibootAddressSection {
            kernel_start,
            kernel_end,
            multiboot_start,
            multiboot_end,
        }
    }
}
