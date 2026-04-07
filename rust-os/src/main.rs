// MIT License / Apache-2.0
// Copyright (c) 2024 Rust OS Developers

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use volatile::Volatile;
use spin::Mutex;
use x86_64::instructions::interrupts;

/// VGA text mode buffer address
const VGA_BUFFER_ADDRESS: usize = 0xb8000;
/// Number of rows in VGA text mode
const VGA_ROWS: usize = 25;
/// Number of columns in VGA text mode
const VGA_COLS: usize = 80;

/// Represents a single character cell in VGA text mode
#[repr(transparent)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// Color codes for VGA text mode
#[derive(Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new ColorCode from foreground and background colors
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Available colors for VGA text mode
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy)]
enum Color {
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

/// VGA buffer structure
struct Buffer {
    chars: [[Volatile<ScreenChar>; VGA_COLS]; VGA_ROWS],
}

/// Writer type for printing to VGA buffer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Write a byte to the screen (handles newlines)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_COLS {
                    self.new_line();
                }

                let row = VGA_ROWS - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Write a string slice to the screen
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Non-printable ASCII: print replacement character
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Scroll the screen up by one line
    fn new_line(&mut self) {
        for row in 1..VGA_ROWS {
            for col in 0..VGA_COLS {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(VGA_ROWS - 1);
        self.column_position = 0;
    }

    /// Clear a single row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..VGA_COLS {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

/// Implementation of core::fmt::Write for Writer
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global writer instance wrapped in a mutex for thread safety
static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
});

/// Macro for printing to VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::vga_buffer::_print(format_args!($($arg)*));
    }};
}

/// Macro for printing with newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Internal print function used by macros
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

/// Panic handler - prints panic message and halts
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n\n{}", info);
    loop {}
}

/// Kernel entry point called by bootloader
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Test basic output
    println!("Welcome to RustOS!");
    println!("==================");
    println!();
    println!("Boot successful!");
    println!("VGA text mode initialized.");
    println!();
    
    // Print some colored output to verify color support
    print!("This is ");
    WRITER.lock().color_code = ColorCode::new(Color::LightBlue, Color::Black);
    print!("blue ");
    WRITER.lock().color_code = ColorCode::new(Color::Yellow, Color::Black);
    print!("yellow ");
    WRITER.lock().color_code = ColorCode::new(Color::LightRed, Color::Black);
    print!("red ");
    WRITER.lock().color_code = ColorCode::new(Color::LightGreen, Color::Black);
    println!("text!");
    println!();
    
    // Demonstrate scrolling
    println!("Testing screen scroll...");
    for i in 0..20 {
        println!("Line {} of scroll test", i);
    }
    
    println!();
    println!("RustOS Phase 1: Bootstrapping complete!");
    println!("Press Ctrl+C in QEMU to exit.");
    
    // Halt CPU (infinite loop for now)
    loop {}
}
