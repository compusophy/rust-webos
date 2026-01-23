use wasm_bindgen::prelude::*;
use std::sync::Once;
use std::collections::VecDeque;
use std::cell::RefCell;

mod gfx;
mod hw;
mod sys;

mod term;

mod bios;
pub mod kernel;

static INIT: Once = Once::new();

// Global Machine State
thread_local! {
    static MACHINE: RefCell<Option<kernel::Machine>> = RefCell::new(None);
    static INPUT_QUEUE: RefCell<Option<VecDeque<String>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn init_os() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        INPUT_QUEUE.with(|q| {
            *q.borrow_mut() = Some(VecDeque::new());
        });
        
        let machine = kernel::Machine::new();

        MACHINE.with(|m| {
            *m.borrow_mut() = Some(machine);
        });
        
        web_sys::console::log_1(&"virtual machine initialized".into());
    });
}

#[wasm_bindgen]
pub fn tick() {
    MACHINE.with(|m| {
        let mut borrow = m.borrow_mut();
        if let Some(machine) = borrow.as_mut() {
            let now = web_sys::window().unwrap().performance().unwrap().now();
            let frame_time = now - machine.last_time;
            machine.last_time = now;
            
            // FPS Counter
            if now - machine.last_sec_time >= 1000.0 {
                machine.real_fps = machine.frames_buffer as f64;
                machine.frames_buffer = 0;
                machine.last_sec_time = now;
            }
            
            machine.accumulator += frame_time;

            // Fixed timestep: 60 ticks per second (16.66ms per tick)
            const TICK_RATE: f64 = 1000.0 / 60.0; 
            const MAX_STEPS_PER_FRAME: i32 = 10;
            
            let mut steps = 0;
            while machine.accumulator >= TICK_RATE && steps < MAX_STEPS_PER_FRAME {
                let mut input_op = None;
                INPUT_QUEUE.with(|q| {
                    if let Some(queue) = q.borrow_mut().as_mut() {
                        input_op = queue.pop_front();
                    }
                });
                
                machine.step(input_op);
                machine.accumulator -= TICK_RATE;
                steps += 1;
            }

            // Render always happens once per browser frame
            // Map text mode to GPU VRAM (conceptually)
            if !machine.gui_mode {
                machine.term.render(&mut machine.bus.gpu, 4, 0);
            }
        }
    });
}

#[wasm_bindgen]
pub fn get_video_buffer_ptr() -> *const u8 {
    MACHINE.with(|m| {
        if let Some(machine) = m.borrow().as_ref() {
            machine.bus.gpu.buffer.as_ptr()
        } else {
            std::ptr::null()
        }
    })
}

#[wasm_bindgen]
pub fn on_keydown(key: String, _ctrl: bool, _alt: bool, _meta: bool) {
    INPUT_QUEUE.with(|q| {
        if let Some(queue) = q.borrow_mut().as_mut() {
            queue.push_back(key);
        }
    });
}

#[wasm_bindgen]
pub fn on_keyup(_key: String) {
    // Optional
}
