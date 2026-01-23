# Rust WebOS (wasmix)

A minimal "Web OS" virtual machine environment running in the browser via Rust and WebAssembly.
This project simulates a complete 32-bit computer architecture in software, providing a retro unix-like experience with a functioning shell, filesystem, and BIOS.

## Features

- **Virtual Hardware**:
  - **CPU**: Simulated 32-bit RISC-like processor architecture.
  - **RAM**: 16 MB Linear Memory simulation.
  - **GPU**: 512x512 RGBA Video RAM (1 MB) with pixel-perfect rendering.
- **System**:
  - **BIOS**: Authentic boot sequence with RAM check, hardware detection, and POST.
  - **Filesystem**: In-memory Virtual Filesystem (VFS) with directory support.
  - **Persistence**: Automatically saves filesystem state to browser `localStorage`.
- **Interface**:
  - **Terminal**: Custom-built shell with command history (Up/Down arrows) and support for colored output.
  - **Shell**: Unix-like command structure.

## Architecture

The system is designed as a modular **32-bit Virtual Machine** (`src/hw/`) running a Rust-based Kernel (`src/lib.rs`).

1. **Hardware Layer**: Simulated hardware components (CPU, Bus, RAM, GPU).
2. **BIOS Layer**: Handles startup, memory testing, and basic input/output before handing off to the kernel.
3. **Kernel Layer**: Manages resources, filesystem, and shell execution.
4. **Userland**: The Shell environment where users interact with the system.

## Commands

The following commands are implemented in the Shell firmware:

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `clear` | Clear screen |
| `ls` | List files in current directory |
| `cd <path>` | Change directory (supports `..` and absolute paths) |
| `mkdir <name>` | Create a new directory |
| `touch <name>` | Create an empty file |
| `df` | Show Disk Usage statistics |
| `sysinfo`| Display System Hardware Information |
| `monitor`| Show Real-time System Monitor (Hz, RAM, VRAM) |
| `uptime` | Show system uptime |
| `date` | Show Real World Time |
| `reboot` | Soft Reboot the system |
| `reset` | **Factory Reset**: Wipe all data and restore to default |

## Keyboard Shortcuts

- **Arrow Up/Down**: Navigate command history.
- **Enter**: Execute command.
- **Backspace**: Delete character.

## Build & Run

### Prerequisites
- **Rust Toolchain**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Trunk**: WASM web application bundler.
  ```bash
  cargo install trunk
  ```

### Development
Run the local development server. This will watch for changes and auto-reload.
```bash
trunk serve --open
```

### Build for Release
Build the optimized WASM bundle for deployment.
```bash
trunk build --release
```

## Project Structure

- `src/hw/`: Hardware simulation (CPU, RAM, GPU, Bus).
- `src/sys/`: System software (Filesystem, Shell).
- `src/bios.rs`: Boot logic and POST sequence.
- `src/term/`: Terminal emulator and rendering logic.
- `src/lib.rs`: Kernel entry point and main loop.

## License
MIT
