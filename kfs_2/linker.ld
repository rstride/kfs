ENTRY(start)

SECTIONS {
    . = 0x0000800;

    .gdt :
    {
        KEEP(*(.gdt))
    }

    .boot :
    {
        KEEP(*(.multiboot_header))
    }

    . = 2M;

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
        *(.data.rel.ro .data.rel.ro.*)
    }

    .data :
    {
        *(.data .data.*)
        *(.got .got.*)
        *(.got.plt .got.plt.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }
}
