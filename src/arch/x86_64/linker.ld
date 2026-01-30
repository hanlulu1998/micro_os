ENTRY(start)

SECTIONS {
  . = 1M;

  .rodata :
  {
    /* ensure that the multiboot header is at the beginning */
    KEEP(*(.multiboot_header))
    *(.rodata .rodata.*)
    . = ALIGN(4K);
  }

  . = ALIGN(4K);

  .text :
  {
    *(.text .text.*)
    . = ALIGN(4K);
  }

  . = ALIGN(4K);

  .data :
  {
    *(.data .data.*)
    . = ALIGN(4K);
  }

  . = ALIGN(4K);

  .bss :
  {
    *(.bss .bss.*)
    . = ALIGN(4K);
  }

  . = ALIGN(4K);

  .linkme :
  {
    __start_linkm2_TEST_REGISTRY = .;
    KEEP(*("linkm2_TEST_REGISTRY"))
    __stop_linkm2_TEST_REGISTRY = .;

    __start_linkme_TEST_REGISTRY = .;
    KEEP(*("linkme_TEST_REGISTRY"))
    __stop_linkme_TEST_REGISTRY = .;
    . = ALIGN(4K);
  }

  . = ALIGN(4K);

  .debug :
  {
    *(.debug*  .comment)
    . = ALIGN(4K);
  }

  /DISCARD/ :
  {
    *(.eh_frame)
    *(.eh_frame_hdr)
  }

}