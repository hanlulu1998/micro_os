use crate::{
    assert_has_not_been_called,
    memory::{
        allocator::{HEAP_ALLOCATOR, HEAP_SIZE, HEAP_START},
        area_frame_allocator::AreaFrameAllocator,
        paging::{EntryFlags, Page, PhysicalAddress},
    },
    multiboot_info::MultibootInfo,
};

pub mod allocator;
pub mod area_frame_allocator;
pub mod paging;
pub mod stack_allocator;

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

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let target = self.start.number.checked_add(n)?;
        if target > self.end.number {
            self.start.number = self.end.number + 1;
            return None;
        }

        let result = Frame { number: target };
        self.start.number = target + 1;
        Some(result)
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub fn init<'a>(boot_info: &'a MultibootInfo) -> MemoryController<'a> {
    assert_has_not_been_called!("memory::init must be called only once");

    use crate::{memory::paging::remap_the_kernel, utils::x86_64_control};

    let mut frame_allocator = AreaFrameAllocator::from_multiboot_info(boot_info);

    x86_64_control::enable_nxe_bit();
    x86_64_control::enable_write_protect_bit();
    let mut active_table = remap_the_kernel(&mut frame_allocator, boot_info);

    // Initialize the heap
    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }

    // Initialize the heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    };

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_alloc_start, stack_alloc_end);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    }
}

pub use self::stack_allocator::Stack;

pub struct MemoryController<'a> {
    active_table: paging::ActivePageTable,
    frame_allocator: area_frame_allocator::AreaFrameAllocator<'a>,
    stack_allocator: stack_allocator::StackAllocator,
}

impl<'a> MemoryController<'a> {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController {
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator,
        } = self;
        stack_allocator.alloc_stack(active_table, frame_allocator, size_in_pages)
    }
}
