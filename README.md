# Rust WebOS

A minimal "Web OS" terminal environment running in the browser via Rust and WebAssembly.

## Features
- **Pixel-perfect Rendering**: Custom 512x512 pixel buffer using HTML Canvas.
- **Terminal Emulator**: Built-in text rendering (font8x8), scrolling, and cursor support.
- **Virtual Filesystem**: 
  - Directory navigation (`cd`, `ls`, `mkdir`).
  - Disk usage tracking (`df`).
  - In-memory storage (non-persistent in this version).
- **Shell**: Custom command interpreter handling input buffers and basic commands.
- **Minimal Dependencies**: Uses `wasm-bindgen` and `web-sys` for glue, but core logic is pure Rust.

## Build & Run

1. **Prerequisites**:
   - Rust toolchain
   - `wasm-pack`

2. **Build**:
   ```bash
   wasm-pack build --target web
   ```

3. **Run**:
   ```bash
   python -m http.server 8080 --bind 127.0.0.1
   ```
   Open [http://127.0.0.1:8080](http://127.0.0.1:8080).
