# Rust WebOS (wasmix)

A minimal "Web OS" virtual machine environment running in the browser via Rust and WebAssembly.
This project simulates a complete 32-bit computer architecture in software, providing a retro unix-like experience with a functioning shell, filesystem, and BIOS.

**Now featuring a User Space Desktop Environment (GUI) and Dynamic WASM Executable Support!**

## Features

- **Virtual Hardware**:
  - **CPU**: Simulated 32-bit RISC-like processor architecture.
  - **RAM**: 16 MB Linear Memory simulation.
  - **GPU**: 512x512 RGBA Video RAM (1 MB) with pixel-perfect rendering and syscall support.
- **Kernel Architecture**:
  - **Modular Kernel**: Core OS logic (`src/kernel.rs`) is separated from the browser runtime wrapper.
  - **Multi-Tasking**: Supports switching between Text Mode (Shell) and Graphical Mode (Desktop).
- **WASM Runtime**:
  - **Executable Support**: Compile Rust code to `.wasm` and run it inside the OS.
  - **System Calls**: Programs can print to the terminal, draw to the screen, and check system status.
  - **Preloaded Binaries**:
    - `/bin/hello.wasm`: Simple hello world text app.
    - `/bin/math.wasm`: Math demonstration.
    - `/bin/desktop.wasm`: A full **Graphical User Interface** environment.
- **System**:
  - **BIOS**: Authentic boot sequence with RAM check, hardware detection, and POST.
  - **Filesystem**: In-memory Virtual Filesystem (VFS) with directory support.
  - **Persistence**: Automatically saves filesystem state to browser `localStorage`.
- **Interface**:
  - **Terminal**: Custom-built shell with command history (Up/Down arrows) and support for colored output.
  - **Shell**: Unix-like command structure.

## Architecture

The system is designed as a modular **32-bit Virtual Machine** (`src/hw/`) running a Rust-based Kernel (`src/kernel.rs`).

1. **Hardware Layer**: Simulated hardware components (CPU, Bus, RAM, GPU).
2. **BIOS Layer**: Handles startup, memory testing, and basic input/output before handing off to the kernel.
3. **Kernel Layer**: Manages resources, filesystem, and shell execution.
4. **WASM Runtime**: A sandbox (`wasmi`) that loads and executes userspace `.wasm` binaries.
5. **Userland**: The Shell and GUI Desktop environment where users interact with the system.

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
| `exec <path>` | **Run a WASM executable** (e.g. `exec /bin/desktop.wasm`) |
| `df` | Show Disk Usage statistics |
| `sysinfo`| Display System Hardware Information and Real-time Status |
| `uptime` | Show system uptime |
| `date` | Show Real World Time |
| `reboot` | Soft Reboot the system |
| `reset` | **Factory Reset**: Wipe all data and restore to default |

## Graphical User Interface (GUI)

The OS supports a full Graphical Mode. To launch the Desktop Environment:

```bash
exec /bin/desktop.wasm
```

This launches a user-space WASM application that uses the `sys_enable_gui_mode` system call to take control of the video buffer, rendering a window manager, taskbar, and applications.

## Build & Run

### Prerequisites
- **Rust Toolchain**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Trunk**: WASM web application bundler.
  ```bash
  cargo install trunk
  ```
- **Wasm32 Target**:
  ```bash
  rustup target add wasm32-unknown-unknown
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
- `src/sys/`: System software (Filesystem, Shell, WASM Runtime).
- `src/kernel.rs`: Main Kernel logic and State Machine.
- `src/bios.rs`: Boot logic and POST sequence.
- `src/term/`: Terminal emulator and rendering logic.
- `src/lib.rs`: WASM Bindgen bridge (Browser Interface).
- `apps/`: Source code for user-space WASM applications (`hello`, `math`, `desktop`).

## Version

**v0.2.0** - desktop terminal now starts in `/local/user` for cleaner ux, fixed refcell borrow panic in nested wasm exec.

## License
MIT
