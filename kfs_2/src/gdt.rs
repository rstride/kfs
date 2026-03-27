use core::arch::asm;

const GDT_ENTRIES: usize = 7; // Null, KC, KD, KS, UC, UD, US

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access_byte: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(base: u32, limit: u32, access_byte: u8, flags: u8) -> Self {
        let limit_low = (limit & 0xFFFF) as u16;
        let limit_high = ((limit >> 16) & 0x0F) as u8;
        let flags_limit_high = (flags << 4) | limit_high;

        GdtEntry {
            limit_low,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access_byte,
            flags_limit_high,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

// Global variable holding our entries
#[link_section = ".gdt"]
#[used]
static mut GDT: [GdtEntry; GDT_ENTRIES] = [
    // 0: Null descriptor
    GdtEntry::new(0, 0, 0, 0),
    // 1: Kernel Code: Base 0, Limit 0xFFFFFFFF, Access 0x9A, Flags 0xC
    GdtEntry::new(0, 0xFFFFF, 0x9A, 0xC),
    // 2: Kernel Data: Base 0, Limit 0xFFFFFFFF, Access 0x92, Flags 0xC
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xC),
    // 3: Kernel Stack: Based on subject, we create it just like Data for flat model but could be specific base/limit if needed. Here flat model fits our kernel stack in .bss.
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xC),
    // 4: User Code: Base 0, Limit 0xFFFFFFFF, Access 0xFA (ring 3), Flags 0xC
    GdtEntry::new(0, 0xFFFFF, 0xFA, 0xC),
    // 5: User Data: Base 0, Limit 0xFFFFFFFF, Access 0xF2 (ring 3), Flags 0xC
    GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC),
    // 6: User Stack: Same as user data.
    GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC),
];

// load GDTR pointing to the linked address (0x800) and reload segment registers
pub unsafe fn init() {
    // 1. Setup GdtDescriptor pointing to GDT address directly mapped by the linker
    let gdtr = GdtDescriptor {
        size: (core::mem::size_of::<[GdtEntry; GDT_ENTRIES]>() - 1) as u16,
        offset: core::ptr::addr_of!(GDT) as u32,
    };

    // 3. Load GDT utilizing inline assembly
    asm!(
        "lgdt [{}]",
        in(reg) &gdtr as *const _,
        options(readonly, nostack, preserves_flags)
    );

    // 4. Reload segment registers.
    // Kernel Code segment selector: 0x08 (Index 1)
    // Kernel Data/Stack segment selector: 0x10 (Index 2)
    asm!(
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ax, 0x18", // You could use index 3 for SS (0x18) if we want different segments, let's use 0x18 for Kernel Stack entry
        "mov ss, ax",
        "push 0x08",
        "lea eax, [2f]",
        "push eax",
        "retf",
        "2:",
        out("eax") _, // Clobbering eax
        options(nostack)
    );
}

