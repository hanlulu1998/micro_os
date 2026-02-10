use core::panic;

use crate::multiboot_info::{MultibootAddressSection, MultibootInfo, MultibootMemMapEntry};

use super::{Frame, FrameAllocator};

pub const MAX_FREE_FRAMES: usize = 1024; // 可调，根据内存大小
static FREE_FRAME_LIST: spin::Mutex<[Option<usize>; MAX_FREE_FRAMES]> =
    spin::Mutex::new([None; MAX_FREE_FRAMES]);
pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,

    areas: &'a [MultibootMemMapEntry],
    current_area: Option<&'a MultibootMemMapEntry>,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
    free_count: usize, // 当前 free_list 中帧数量
}

impl<'a> FrameAllocator for AreaFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if self.free_count > 0 {
            self.free_count -= 1;
            let frame_number = FREE_FRAME_LIST.lock()[self.free_count].take().unwrap();
            return Some(Frame {
                number: frame_number,
            });
        }

        if let Some(area) = self.current_area {
            let frame = Frame {
                number: self.next_free_frame.number,
            };

            let current_area_last_frame = {
                let address = area.base_addr as usize + area.length as usize - 1;
                Frame::containing_address(address)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1,
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1,
                };
            } else {
                self.next_free_frame = Frame {
                    number: frame.number + 1,
                };
                return Some(frame);
            }
            self.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        if self.free_count < MAX_FREE_FRAMES {
            FREE_FRAME_LIST.lock()[self.free_count] = Some(frame.number);
            self.free_count += 1;
        } else {
            panic!("Too many free frames");
        }
    }
}

impl<'a> AreaFrameAllocator<'a> {
    pub fn choose_next_area(&mut self) {
        self.current_area = self
            .areas
            .iter()
            .filter(|area| {
                let address = area.base_addr as usize + area.length as usize - 1;
                Frame::containing_address(address) >= self.next_free_frame
            })
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }

    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: &'a [MultibootMemMapEntry],
    ) -> Self {
        let mut allocator = AreaFrameAllocator::<'a> {
            next_free_frame: Frame::containing_address(0),
            areas: memory_areas,
            current_area: None,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
            free_count: 0,
        };
        allocator.choose_next_area();
        allocator
    }

    pub fn from_multiboot_address_sections(
        multiboot_address_sections: &'a MultibootAddressSection,
        memory_areas: &'a [MultibootMemMapEntry],
    ) -> Self {
        AreaFrameAllocator::<'a>::new(
            multiboot_address_sections.kernel_start,
            multiboot_address_sections.kernel_end,
            multiboot_address_sections.multiboot_start,
            multiboot_address_sections.multiboot_end,
            memory_areas,
        )
    }

    pub fn from_multiboot_info(boot_info: &'a MultibootInfo) -> Self {
        let address_sections = boot_info.get_multiboot_address_section();
        let memory_entries = boot_info.get_memory_entries();
        AreaFrameAllocator::<'a>::new(
            address_sections.kernel_start,
            address_sections.kernel_end,
            address_sections.multiboot_start,
            address_sections.multiboot_end,
            memory_entries,
        )
    }
}
