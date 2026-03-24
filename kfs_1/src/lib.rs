#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kmain(_magic: u32, _info: u32) {
    // Clear all terminals and present welcome msg
    for i in 0..vga_buffer::NUM_SCREENS {
        vga_buffer::switch_screen(i);
        vga_buffer::clear_screen();
        crate::println!("[Virtual Terminal {}]", i + 1);
    }
    
    vga_buffer::switch_screen(0);
    crate::println!("42");
    crate::println!("Welcome to KFS-1!");
    crate::println!("Use F1, F2, F3 to switch virtual terminals.");
    crate::print!("> ");

    let mut last_scancode = 0;
    loop {
        let scancode = unsafe { vga_buffer::inb(0x60) };
        if scancode != last_scancode {
            if (scancode & 0x80) == 0 {
                // Key pressed
                match scancode {
                    0x3B => vga_buffer::switch_screen(0), // F1
                    0x3C => vga_buffer::switch_screen(1), // F2
                    0x3D => vga_buffer::switch_screen(2), // F3
                    0x48 | 0x49 => vga_buffer::scroll_up(), // Arrow Up / Page Up
                    0x50 | 0x51 => vga_buffer::scroll_down(), // Arrow Down / Page Down
                    _ => {
                        let c = match scancode {
                            0x02 => '1', 0x03 => '2', 0x04 => '3', 0x05 => '4', 0x06 => '5',
                            0x07 => '6', 0x08 => '7', 0x09 => '8', 0x0A => '9', 0x0B => '0',
                            0x1E => 'a', 0x30 => 'b', 0x2E => 'c', 0x20 => 'd', 0x12 => 'e',
                            0x21 => 'f', 0x22 => 'g', 0x23 => 'h', 0x17 => 'i', 0x24 => 'j',
                            0x25 => 'k', 0x26 => 'l', 0x32 => 'm', 0x31 => 'n', 0x18 => 'o',
                            0x19 => 'p', 0x10 => 'q', 0x13 => 'r', 0x1F => 's', 0x14 => 't',
                            0x16 => 'u', 0x2F => 'v', 0x11 => 'w', 0x2D => 'x', 0x15 => 'y',
                            0x2C => 'z', 0x39 => ' ', 0x1C => '\n',
                            0x0E => '\x08', // Backspace (We won't process it fully here, just a char)
                            _ => '?',
                        };
                        if c != '?' {
                            crate::print!("{}", c);
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
