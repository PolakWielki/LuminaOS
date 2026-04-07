# RustOS - Phase 1: Bootstrapping and Minimal Kernel

## Overview

This is Phase 1 of the RustOS project - a minimal but real operating system written in Rust.

### What's Implemented

- **No-std Rust environment**: The kernel runs without the standard library
- **VGA text mode driver**: Direct hardware access to display text on screen
- **Panic handler**: Graceful handling of kernel panics with error output
- **Bootloader integration**: Uses the `bootloader` crate for x86_64 boot
- **Basic printing macros**: `print!` and `println!` macros for kernel output

### Architecture

```
┌─────────────────────────────────────┐
│         Bootloader (crate)          │
│   - Loads kernel into memory        │
│   - Sets up initial page tables     │
│   - Jumps to _start()               │
└─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│         Kernel Entry (_start)       │
│   - Initialize VGA buffer           │
│   - Print welcome message           │
│   - Enter main loop                 │
└─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│         VGA Text Mode Driver        │
│   - Write characters to 0xb8000     │
│   - Handle scrolling                │
│   - Support colors                  │
└─────────────────────────────────────┘
```

## Directory Structure

```
rust-os/
├── Cargo.toml              # Package manifest with dependencies
├── .cargo/
│   └── config.toml         # Build configuration (build-std, bootloader)
├── src/
│   └── main.rs             # Kernel entry point and VGA driver
└── README.md               # This file
```

## Building

### Prerequisites

1. **Rust nightly toolchain** (required for `no_std` and unstable features):
   ```bash
   rustup install nightly
   rustup default nightly
   rustup component add rust-src
   ```

2. **QEMU** for testing:
   ```bash
   # Ubuntu/Debian
   sudo apt install qemu-system-x86
   
   # macOS
   brew install qemu
   
   # Windows (with Chocolatey)
   choco install qemu
   ```

### Build Commands

```bash
cd rust-os

# Build the kernel
cargo build --release

# Run in QEMU
cargo run
```

The `cargo run` command will:
1. Build the kernel with the bootloader
2. Create a bootable disk image
3. Launch QEMU with the image

## Expected Output

When you run the kernel in QEMU, you should see:

```
Welcome to RustOS!
==================

Boot successful!
VGA text mode initialized.

This is blue yellow red text!

Testing screen scroll...
Line 0 of scroll test
Line 1 of scroll test
...
Line 19 of scroll test

RustOS Phase 1: Bootstrapping complete!
Press Ctrl+C in QEMU to exit.
```

The text should appear in different colors as indicated.

## Technical Details

### Memory Layout

- **VGA Buffer**: Located at physical address `0xb8000`
- **Buffer size**: 80 columns × 25 rows = 2000 character cells
- **Each cell**: 2 bytes (1 byte ASCII + 1 byte color code)

### Key Components

#### VGA Text Mode

The VGA text mode buffer is a memory-mapped region where each character cell consists of:
- Byte 0: ASCII character code
- Byte 1: Color code (4 bits foreground, 4 bits background)

#### Panic Handler

The `#[panic_handler]` attribute marks our panic function. When the kernel panics:
1. The panic message is formatted
2. Printed to the VGA buffer
3. CPU enters an infinite loop

#### Print Macros

The `print!` and `println!` macros work similarly to std's versions but write to VGA instead.

### Unsafe Code

The only unsafe block in this phase:

```rust
buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
```

**Justification**: We're creating a mutable reference to a specific memory address (the VGA buffer). This is safe because:
1. The address `0xb8000` is guaranteed by the x86_64 architecture
2. The bootloader has mapped this region as writable
3. Access is synchronized through the `Mutex<Writer>`

## Known Limitations

1. **No interrupt handling**: The kernel runs in a busy loop
2. **No keyboard input**: Output-only at this stage
3. **No memory management**: Uses hardcoded VGA address
4. **Single task**: No multitasking or scheduling
5. **No hardware detection**: Assumes standard VGA text mode

## Next Steps (Phase 2)

Phase 2 will implement memory management:
- Physical memory detection
- Page frame allocator
- Paging and virtual memory setup
- Kernel heap allocator

## Troubleshooting

### "error[E0463]: can't find crate for `core`"

Make sure you have the `rust-src` component installed:
```bash
rustup component add rust-src
```

### QEMU doesn't show output

Try running with explicit VGA:
```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-rustos/release/bootimage-rust-os.bin -vga std
```

### Build fails with linker errors

Ensure you're using the nightly toolchain and have `rust-src` installed.

## License

MIT License OR Apache-2.0 - See individual source files for details.
