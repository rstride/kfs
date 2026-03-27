use core::fmt;
use core::ptr::write_volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const HISTORY_LINES: usize = 100;

#[derive(Clone, Copy)]
struct ScreenBuffer {
    lines: [[ScreenChar; BUFFER_WIDTH]; HISTORY_LINES],
}

pub const NUM_SCREENS: usize = 3;

pub struct Terminal {
    buffer: ScreenBuffer,
    cursor_row: usize,
    cursor_col: usize,
    view_offset: usize,
    color_code: ColorCode,
}

impl Terminal {
    pub const fn new(color_code: ColorCode) -> Self {
        Terminal {
            buffer: ScreenBuffer {
                lines: [[ScreenChar {
                    ascii_character: b' ',
                    color_code,
                }; BUFFER_WIDTH]; HISTORY_LINES],
            },
            cursor_row: 0,
            cursor_col: 0,
            view_offset: 0,
            color_code,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        // Reset scroll position when actively typing
        if self.cursor_row >= BUFFER_HEIGHT {
            self.view_offset = self.cursor_row - BUFFER_HEIGHT + 1;
        }

        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.cursor_col >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.lines[self.cursor_row][self.cursor_col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.cursor_col += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.cursor_row < HISTORY_LINES - 1 {
            self.cursor_row += 1;
        } else {
            // Shift history up (lose the topmost line)
            for row in 1..HISTORY_LINES {
                self.buffer.lines[row - 1] = self.buffer.lines[row];
            }
            self.clear_row(HISTORY_LINES - 1);
        }

        self.cursor_col = 0;

        // Ensure view tracks the bottom
        if self.cursor_row >= BUFFER_HEIGHT {
            self.view_offset = self.cursor_row - BUFFER_HEIGHT + 1;
        } else {
            self.view_offset = 0;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.lines[row][col] = blank;
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
        self.update_cursor();
    }

    fn update_cursor(&self) {
        if self.cursor_row >= self.view_offset && self.cursor_row < self.view_offset + BUFFER_HEIGHT {
            let screen_row = self.cursor_row - self.view_offset;
            let pos = screen_row * BUFFER_WIDTH + self.cursor_col;
            unsafe {
                outb(0x3D4, 14);
                outb(0x3D5, (pos >> 8) as u8);
                outb(0x3D4, 15);
                outb(0x3D5, (pos & 0xFF) as u8);
            }
        } else {
            // Cursor is currently out of the scrolling view area (hide it)
            let pos = BUFFER_HEIGHT * BUFFER_WIDTH;
            unsafe {
                outb(0x3D4, 14);
                outb(0x3D5, (pos >> 8) as u8);
                outb(0x3D4, 15);
                outb(0x3D5, (pos & 0xFF) as u8);
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.view_offset > 0 {
            self.view_offset -= 1;
            self.update_cursor();
        }
    }

    pub fn scroll_down(&mut self) {
        if self.cursor_row >= BUFFER_HEIGHT {
            let max_offset = self.cursor_row - BUFFER_HEIGHT + 1;
            if self.view_offset < max_offset {
                self.view_offset += 1;
                self.update_cursor();
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 2 { // Keep the "> " prompt
            self.cursor_col -= 1;
            self.buffer.lines[self.cursor_row][self.cursor_col] = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            self.update_cursor();
        }
    }
}

pub fn backspace() {
    let active = unsafe { ACTIVE_SCREEN };
    let term = unsafe { &mut TERMINALS[active] };
    term.backspace();
    flush_screen();
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static mut ACTIVE_SCREEN: usize = 0;
static mut TERMINALS: [Terminal; NUM_SCREENS] = [
    Terminal::new(ColorCode::new(Color::White, Color::Black)),
    Terminal::new(ColorCode::new(Color::LightGreen, Color::Black)),
    Terminal::new(ColorCode::new(Color::LightCyan, Color::Black)),
];

pub fn switch_screen(idx: usize) {
    if idx < NUM_SCREENS {
        unsafe { ACTIVE_SCREEN = idx };
        flush_screen();
    }
}

pub fn scroll_up() {
    let active = unsafe { ACTIVE_SCREEN };
    let term = unsafe { &mut TERMINALS[active] };
    term.scroll_up();
    flush_screen();
}

pub fn scroll_down() {
    let active = unsafe { ACTIVE_SCREEN };
    let term = unsafe { &mut TERMINALS[active] };
    term.scroll_down();
    flush_screen();
}

#[repr(transparent)]
struct VgaBuffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub fn flush_screen() {
    let active = unsafe { ACTIVE_SCREEN };
    let vga = unsafe { &mut *(0xb8000 as *mut VgaBuffer) };
    let term = unsafe { &TERMINALS[active] };
    
    let offset = term.view_offset;

    for row in 0..BUFFER_HEIGHT {
        for col in 0..BUFFER_WIDTH {
            unsafe {
                write_volatile(&mut vga.chars[row][col], term.buffer.lines[offset + row][col]);
            }
        }
    }
    term.update_cursor();
}

pub fn clear_screen() {
    let active = unsafe { ACTIVE_SCREEN };
    let term = unsafe { &mut TERMINALS[active] };
    for row in 0..HISTORY_LINES {
        term.clear_row(row);
    }
    term.cursor_row = 0;
    term.cursor_col = 0;
    term.view_offset = 0;
    flush_screen();
}

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let active = unsafe { ACTIVE_SCREEN };
    let term = unsafe { &mut TERMINALS[active] };
    term.write_fmt(args).unwrap();
    flush_screen();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
