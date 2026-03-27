use crate::vga_buffer;
use core::arch::asm;

const CMD_BUFFER_SIZE: usize = 128;

pub struct Shell {
    buffer: [u8; CMD_BUFFER_SIZE],
    idx: usize,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            buffer: [0; CMD_BUFFER_SIZE],
            idx: 0,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if c == '\n' {
            self.execute_command();
            self.reset_prompt();
        } else if c == '\x08' {
            // Backspace
            if self.idx > 0 {
                self.idx -= 1;
                self.buffer[self.idx] = 0;
                vga_buffer::backspace();
            }
        } else if self.idx < CMD_BUFFER_SIZE {
            self.buffer[self.idx] = c as u8;
            self.idx += 1;
            crate::print!("{}", c);
        }
    }

    fn execute_command(&mut self) {
        crate::print!("\n");
        let mut len = self.idx;
        while len > 0 && self.buffer[len - 1] == b' ' {
            len -= 1; // pure trim right
        }
        let mut start = 0;
        while start < len && self.buffer[start] == b' ' {
            start += 1;
        }

        let cmd_bytes = &self.buffer[start..len];
        if cmd_bytes.is_empty() {
            return;
        }

        if cmd_bytes == b"halt" {
            crate::println!("Halting CPU...");
            unsafe { asm!("cli", "hlt"); }
        } else if cmd_bytes == b"reboot" {
            crate::println!("Rebooting...");
            unsafe { vga_buffer::outb(0x64, 0xFE); }
        } else if cmd_bytes == b"stack" || cmd_bytes == b"print_stack" {
            print_kernel_stack();
        } else {
            crate::println!("Unknown command.");
        }
    }

    pub fn reset_prompt(&mut self) {
        self.idx = 0;
        self.buffer.fill(0);
        crate::print!("> ");
    }
}

pub fn print_kernel_stack() {
    let mut esp: u32;
    let mut ebp: u32;

    unsafe {
        asm!("mov {}, esp", out(reg) esp);
        asm!("mov {}, ebp", out(reg) ebp);
    }

    crate::println!("Kernel Stack Dump:");
    crate::println!("ESP: 0x{:08X}", esp);
    crate::println!("EBP: 0x{:08X}", ebp);

    // Provide a human-readable dump up to EBP, maximum 16 items
    let mut ptr = esp as *const u32;
    let ebp_ptr = ebp as *const u32;

    crate::println!("--- Stack Content ---");
    let mut count = 0;
    while ptr <= ebp_ptr && count < 16 {
        unsafe {
            let val = *ptr;
            crate::println!("[0x{:08X}] : 0x{:08X}", ptr as u32, val);
            ptr = ptr.add(1);
        }
        count += 1;
    }
    crate::println!("---------------------");
}
