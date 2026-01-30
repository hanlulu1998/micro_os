use crate::{
    memory::{
        FrameAllocator,
        area_frame_allocator::AreaFrameAllocator,
        paging::{ActivePageTable, EntryFlags, Page},
    },
    multiboot_info::MultibootInfo,
    serial_println,
};

pub fn test_paging(multiboot_information_address: usize) {
    let boot_info = MultibootInfo::new(multiboot_information_address);

    let address_sections = boot_info.get_multiboot_address_section();

    let memory_entries = boot_info.get_memory_entries();
    let mut allocator =
        AreaFrameAllocator::from_multiboot_address_sections(&address_sections, memory_entries);

    let mut page_table = ActivePageTable::new();

    let addr = 42 * 512 * 512 * 4096 + 42 * 4096;
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frames");

    serial_println!(
        "None = {:?}, map to {:?}",
        page_table.translate(addr),
        frame
    );

    page_table.map_to(page, frame, EntryFlags::empty(), &mut allocator);
    serial_println!("Some = {:?}", page_table.translate(addr));
    serial_println!("next free frame: {:?}", allocator.allocate_frame());

    let map_address = unsafe { *(Page::containing_address(addr).start_address() as *const u64) };
    serial_println!("{:#x}", map_address);

    page_table.unmap(Page::containing_address(addr), &mut allocator);
    serial_println!("None = {:?}", page_table.translate(addr));
}


pub fn test_remap_the_kernel(multiboot_information_address: usize) {
    use crate::{memory::paging::remap_the_kernel, println, utils::x86_64_control};

    let boot_info = MultibootInfo::new(multiboot_information_address);

    let address_sections = boot_info.get_multiboot_address_section();

    println!(
        "kernel start: 0x{:x}, kernel end: 0x{:x}",
        address_sections.kernel_start, address_sections.kernel_end
    );

    println!(
        "multiboot start: 0x{:x}, multiboot end: 0x{:x}",
        address_sections.multiboot_start, address_sections.multiboot_end
    );

    let memory_entries = boot_info.get_memory_entries();
    let mut frame_allocator =
        AreaFrameAllocator::from_multiboot_address_sections(&address_sections, memory_entries);

    x86_64_control::enable_nxe_bit();
    x86_64_control::enable_write_protect_bit();
    remap_the_kernel(&mut frame_allocator, &boot_info);

    frame_allocator.allocate_frame();

    println!("It did not crash!");
}
