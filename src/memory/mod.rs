use crate::{
    assert_has_not_been_called,
    memory::{
        allocator::{HEAP_ALLOCATOR, HEAP_SIZE, HEAP_START},
        area_frame_allocator::AreaFrameAllocator,
        paging::{EntryFlags, Page, PhysicalAddress},
    },
};

pub mod allocator;
pub mod area_frame_allocator;
pub mod paging;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

pub const PAGE_SIZE: usize = 4096;

impl Frame {
    fn containing_address(address: PhysicalAddress) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init(multiboot_information_address: usize) {
    assert_has_not_been_called!("memory::init must be called only once");

    use crate::{memory::paging::remap_the_kernel, utils::x86_64_control};

    let boot_info = crate::multiboot_info::MultibootInfo::new(multiboot_information_address);

    let address_sections = boot_info.get_multiboot_address_section();

    let memory_entries = boot_info.get_memory_entries();
    let mut frame_allocator =
        AreaFrameAllocator::from_multiboot_address_sections(&address_sections, memory_entries);

    x86_64_control::enable_nxe_bit();
    x86_64_control::enable_write_protect_bit();
    let mut active_table = remap_the_kernel(&mut frame_allocator, &boot_info);

    // Initialize the heap
    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    // Initialize the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}
