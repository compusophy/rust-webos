# wasmix (WASM + Unix/Mix)

A minimal "Web OS" virtual machine environment running in the browser via Rust and WebAssembly.

## Architecture
The system is designed as a **32-bit Virtual Machine** (`src/hw/`) running a Rust-based Kernel (`src/lib.rs`).
- **CPU**: Simulated 32-bit processor.
- **RAM**: 16 MB Linear Memory.
- **GPU**: 512x512 RGBA VRAM.
- **FS**: In-memory Virtual Filesystem.

## Commands
The following commands are implemented in the Shell firmware:

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `clear` | Clear screen |
| `ls` | List files |
| `cd` | Change directory (supports wildcards `cd fol*`) |
| `mkdir` | Create directory |
| `df` | Disk Usage |
| `sysinfo`| System Information |
| `reboot` | Reboot system |
| `uptime` | System uptime |
| `date` | Real World Time |
| `monitor`| Real System Monitor (Hz, RAM, VRAM) |

## Build & Run

1. **Prerequisites**:
   - Rust toolchain
   - `trunk` (`cargo install trunk`)

2. **Run**:
   ```bash
   trunk serve --open
   ```

3. **Build Release**:
   ```bash
   trunk build --release
   ```
