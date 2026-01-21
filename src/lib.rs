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

struct OsState {
    gfx: gfx::Context,
    term: term::Terminal,
    shell: shell::Shell,
    fs: fs::FileSystem,
    tick_count: u64,
}

#[wasm_bindgen]
pub fn init_os() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        unsafe {
            INPUT_QUEUE = Some(VecDeque::new());
            
            let gfx = gfx::Context::new(512, 512);
            let mut term = term::Terminal::new(64, 64);
            let mut shell = shell::Shell::new();
            let mut fs = fs::FileSystem::new(10); // 10 MB disk
            
            // Boot sequence
            term.write_str("Welcome to Rust WebOS v0.1\n");
            term.write_str("Initializing kernel...\n");
            term.write_str("Filesystem: Mounted (in-memory, 10MB)\n");
            shell.draw_prompt(&mut term);

            OS_STATE = Some(OsState {
                gfx,
                term,
                shell,
                fs,
                tick_count: 0,
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

    // Process Input
    while let Some(key) = input_queue.pop_front() {
        os.shell.on_key(&key, &mut os.term, &mut os.fs);
    }

    // Render
    os.gfx.clear(0, 0, 0); // Black background
    os.term.render(&mut os.gfx);
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
