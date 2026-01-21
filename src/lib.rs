use wasm_bindgen::prelude::*;
use std::sync::Once;
use std::collections::VecDeque;

mod gfx;
mod fs;
mod term;
mod shell;

static INIT: Once = Once::new();

// Global Kernel State
static mut OS_STATE: Option<OsState> = None;
static mut INPUT_QUEUE: Option<VecDeque<String>> = None;

enum MachineState {
    Post,
    Booting,
    Active,
}

struct OsState {
    gfx: gfx::Context,
    term: term::Terminal,
    shell: shell::Shell,
    fs: fs::FileSystem,
    tick_count: u64,
    state: MachineState,
}

#[wasm_bindgen]
pub fn init_os() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        unsafe {
            INPUT_QUEUE = Some(VecDeque::new());
            
            let gfx = gfx::Context::new(512, 512);
            let mut term = term::Terminal::new(64, 64);
            let shell = shell::Shell::new();
            let fs = fs::FileSystem::new(10); // 10 MB disk
            
            // Initial state is POST
            OS_STATE = Some(OsState {
                gfx,
                term,
                shell,
                fs,
                tick_count: 0,
                state: MachineState::Post,
            });
        }
        
        web_sys::console::log_1(&"OS Initialized".into());
    });
}

#[wasm_bindgen]
pub fn tick() {
    let os = unsafe { OS_STATE.as_mut().expect("OS not initialized") };
    let input_queue = unsafe { INPUT_QUEUE.as_mut().expect("Input queue not initialized") };

    os.tick_count += 1;

    match os.state {
        MachineState::Post => {
            // Simulate BIOS POST
            // Clear screen to Blue basic color or Black
            os.gfx.clear(0, 0, 50); 
            
            // Draw BIOS Text manually or via terminal
            // Let's use terminal but custom position or just write to it.
            // But terminal has state (cursor).
            // Let's use a temporary terminal reset or just draw directly using font.
            // For simplicity, let's use the terminal but reset it often?
            // Actually, let's just write to terminal.
            
            if os.tick_count == 1 {
                os.term.reset();
                os.term.write_str("Rust WebBIOS v1.0\n");
                os.term.write_str("Copyright (C) 2026 CompuSophy Inc.\n\n");
                os.term.write_str("CPU: WASM-32 Virtual Core\n");
            }
            
            if os.tick_count % 10 == 0 && os.tick_count < 100 {
                 let mem = os.tick_count * 1024;
                 let msg = format!("\rMemory Test: {} KB OK", mem);
                 os.term.write_str(&msg);
            }
            
            if os.tick_count > 120 {
                os.state = MachineState::Booting;
                os.term.write_str("\n\nSystem OK.\nBooting from Hard Disk...\n");
            }
            
            // Render
            os.term.render(&mut os.gfx);
        },
        MachineState::Booting => {
            if os.tick_count > 180 {
                // Transition to Active
                os.state = MachineState::Active;
                os.term.reset(); // Clear BIOS screen
                
                // Print Kernel Boot msg
                os.term.write_str("Welcome to Rust WebOS v0.1\n");
                os.term.write_str("Initializing kernel...\n");
                os.term.write_str("Filesystem: Mounted (in-memory, 10MB)\n");
                os.shell.draw_prompt(&mut os.term);
            }
            os.term.render(&mut os.gfx);
        },
        MachineState::Active => {
            // Process Input
            while let Some(key) = input_queue.pop_front() {
                 let res = os.shell.on_key(&key, &mut os.term, &mut os.fs);
                 if res {
                     // Reboot requested (assuming boolean convention for now, wait I need to update shell return type)
                     // Let's hold off on reboot logic in this step or force it by checking a special var?
                     // I will update shell to return bool: true = reboot, false = continue
                     os.state = MachineState::Post;
                     os.tick_count = 0;
                 }
            }
            os.gfx.clear(0, 0, 0); // Black background
            os.term.render(&mut os.gfx);
        }
    }
}

#[wasm_bindgen]
pub fn get_video_buffer_ptr() -> *const u8 {
    let os = unsafe { OS_STATE.as_ref().expect("OS not initialized") };
    os.gfx.buffer.as_ptr()
}

#[wasm_bindgen]
pub fn on_keydown(key: String, _ctrl: bool, _alt: bool, _meta: bool) {
    unsafe {
        if let Some(queue) = INPUT_QUEUE.as_mut() {
            queue.push_back(key);
        }
    }
}

#[wasm_bindgen]
pub fn on_keyup(_key: String) {
    // Optional
}
