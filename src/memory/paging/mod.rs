pub mod entry;
pub mod mapper;
pub mod table;
pub mod temporary_page;

use core::ops::{Deref, DerefMut};

pub use self::entry::*;
use crate::{
    memory::{
        Frame, FrameAllocator, PAGE_SIZE,
        paging::{
            mapper::Mapper,
            table::{Level4, Table},
            temporary_page::TemporaryPage,
        },
    },
    multiboot_info::MultibootInfo,
    println, utils::x86_64_control::{cr3, tlb},
};

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}",
            address
        );
        Page {
            number: address / PAGE_SIZE,
        }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }
}

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub fn new() -> Self {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        self.mapper.p4()
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        self.mapper.p4_mut()
    }

    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut temporary_page::TemporaryPage,
        f: F,
    ) where
        F: FnOnce(&mut Mapper),
    {
        {
            let backup = Frame::containing_address(cr3::read_cr3() as usize);

            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            self.p4_mut()[511].set(
                table.p4_frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
            tlb::tlb_flush_all();

            f(self);

            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::tlb_flush_all();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(cr3::read_cr3() as usize),
        };
        cr3::write_cr3(new_table.p4_frame.start_address() as u64);
        old_table
    }
}

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &MultibootInfo) -> ActivePageTable
where
    A: FrameAllocator,
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);

    let mut active_table = ActivePageTable::new();

    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections = boot_info.get_elf_sections();

        for section in elf_sections.iter().filter(|section| section.size() > 0) {
            if !section.is_allocated() {
                continue;
            }

            assert!(
                section.start_address() % PAGE_SIZE == 0,
                "sections need to be page aligned"
            );

            #[cfg(feature = "use_test")]
            println!(
                "mapping section at addr: {:#x}, size: {:#x}",
                section.sh_addr, section.sh_size
            );

            let flags = Entry::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000); // new
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator); // new

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    #[cfg(feature = "use_test")]
    println!("NEW TABLE!!!");

    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);

    #[cfg(feature = "use_test")]
    println!("guard page at {:#x}", old_p4_page.start_address());

    active_table
}
