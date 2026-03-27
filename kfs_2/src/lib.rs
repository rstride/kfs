#![no_std]
#![no_main]

mod vga_buffer;
mod gdt;
mod shell;

use core::panic::PanicInfo;
use shell::Shell;

#[no_mangle]
pub extern "C" fn kmain(_magic: u32, _info: u32) {
    // 1. Initialize GDT
    crate::println!("Initializing GDT...");
    unsafe {
        gdt::init();
    }
    crate::println!("GDT Initialized and segments reloaded.");

    for i in 0..vga_buffer::NUM_SCREENS {
        vga_buffer::switch_screen(i);
        vga_buffer::clear_screen();
        crate::println!("[Virtual Terminal {}]", i + 1);
    }
    
    vga_buffer::switch_screen(0);
    crate::println!("42");
    crate::println!("Welcome to KFS-2!");
    crate::println!("Use F1, F2, F3 to switch virtual terminals.");
    
    let mut sh = Shell::new();
    sh.reset_prompt();

    let mut last_scancode = 0;
    let mut shift_pressed = false;

    loop {
        let scancode = unsafe { vga_buffer::inb(0x60) };
        if scancode != last_scancode {
            if scancode == 0x2A || scancode == 0x36 {
                shift_pressed = true;
            } else if scancode == 0xAA || scancode == 0xB6 {
                shift_pressed = false;
            } else if (scancode & 0x80) == 0 {
                // Key pressed
                match scancode {
                    0x3B => vga_buffer::switch_screen(0), // F1
                    0x3C => vga_buffer::switch_screen(1), // F2
                    0x3D => vga_buffer::switch_screen(2), // F3
                    0x48 | 0x49 => vga_buffer::scroll_up(), // Arrow Up
                    0x50 | 0x51 => vga_buffer::scroll_down(), // Arrow Down
                    _ => {
                        let (lower, upper) = match scancode {
                            // Row 1
                            0x29 => ('@', '#'),
                            0x02 => ('&', '1'),
                            0x03 => ('\u{82}', '2'), // é
                            0x04 => ('"', '3'),
                            0x05 => ('\'', '4'),
                            0x06 => ('(', '5'),
                            0x07 => ('\u{15}', '6'), // §
                            0x08 => ('\u{8A}', '7'), // è
                            0x09 => ('!', '8'),
                            0x0A => ('\u{87}', '9'), // ç
                            0x0B => ('\u{85}', '0'), // à
                            0x0C => (')', '\u{F8}'), // °
                            0x0D => ('-', '_'),
                            // Row 2
                            0x10 => ('a', 'A'),
                            0x11 => ('z', 'Z'),
                            0x12 => ('e', 'E'),
                            0x13 => ('r', 'R'),
                            0x14 => ('t', 'T'),
                            0x15 => ('y', 'Y'),
                            0x16 => ('u', 'U'),
                            0x17 => ('i', 'I'),
                            0x18 => ('o', 'O'),
                            0x19 => ('p', 'P'),
                            0x1A => ('^', '\u{99}'), // ¨ (rough map)
                            0x1B => ('$', '*'),
                            // Row 3
                            0x1E => ('q', 'Q'),
                            0x1F => ('s', 'S'),
                            0x20 => ('d', 'D'),
                            0x21 => ('f', 'F'),
                            0x22 => ('g', 'G'),
                            0x23 => ('h', 'H'),
                            0x24 => ('j', 'J'),
                            0x25 => ('k', 'K'),
                            0x26 => ('l', 'L'),
                            0x27 => ('m', 'M'),
                            0x28 => ('\u{97}', '%'), // ù
                            0x2B => ('`', '\u{9C}'), // £
                            // Row 4
                            0x56 => ('<', '>'),
                            0x2C => ('w', 'W'),
                            0x2D => ('x', 'X'),
                            0x2E => ('c', 'C'),
                            0x2F => ('v', 'V'),
                            0x30 => ('b', 'B'),
                            0x31 => ('n', 'N'),
                            0x32 => (',', '?'),
                            0x33 => (';', '.'),
                            0x34 => (':', '/'),
                            0x35 => ('=', '+'),
                            // Space and Enter
                            0x39 => (' ', ' '),
                            0x1C => ('\n', '\n'),
                            0x0E => ('\x08', '\x08'), // Backspace
                            _ => ('\0', '\0'),
                        };
                        let c = if shift_pressed { upper } else { lower };
                        if c != '\0' {
                            sh.handle_char(c);
                        }
                    }
                }
            }
            last_scancode = scancode;
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("{}", info);
    loop {}
}
