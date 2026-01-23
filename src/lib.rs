use wasm_bindgen::prelude::*;
use std::sync::Once;
use std::collections::VecDeque;
use std::cell::RefCell;

mod gfx;
mod hw;
mod sys;

mod term;

mod bios;

static INIT: Once = Once::new();

// Global Machine State
thread_local! {
    static MACHINE: RefCell<Option<Machine>> = RefCell::new(None);
    static INPUT_QUEUE: RefCell<Option<VecDeque<String>>> = RefCell::new(None);
}

pub enum MachineState {
    Bios,
    Kernel,
}

pub struct Machine {
    pub cpu: hw::cpu::Cpu,
    pub bus: hw::bus::Bus,
    pub bios: bios::Bios,
    
    // Peripherals / Firmware
    pub term: term::Terminal,
    pub shell: sys::shell::Shell,
    pub fs: sys::fs::FileSystem,
    
    // Timing and State
    tick_count: u64,
    last_time: f64,
    accumulator: f64,
    real_fps: f64,
    frames_buffer: u64,
    pub last_sec_time: f64,
    pub state: MachineState,
}


#[wasm_bindgen]
pub fn init_os() {
    INIT.call_once(|| {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        INPUT_QUEUE.with(|q| {
            *q.borrow_mut() = Some(VecDeque::new());
        });
        
        // Hardware Init
        let ram = hw::ram::Ram::new(16 * 1024 * 1024); // 16 MB RAM
        let gpu = hw::gpu::Gpu::new(512, 512); // VRAM
        let bus = hw::bus::Bus::new(ram, gpu);
        let cpu = hw::cpu::Cpu::new();
        let bios = bios::Bios::new();
        
        // Firmware/Software Init
        // Firmware/Software Init
        let term = term::Terminal::new(64, 32);
        let shell = sys::shell::Shell::new();
        let fs = sys::fs::FileSystem::new(10); // 10 MB disk
        
        // Initial state is POST
        let machine = Machine {
            cpu,
            bus,
            term,
            shell,
            fs,
            tick_count: 0,
            last_time: web_sys::window().unwrap().performance().unwrap().now(),
            accumulator: 0.0,
            real_fps: 0.0,
            frames_buffer: 0,
            last_sec_time: web_sys::window().unwrap().performance().unwrap().now(),
            state: MachineState::Bios,
            bios,
        };

        MACHINE.with(|m| {
            *m.borrow_mut() = Some(machine);
        });
        
        web_sys::console::log_1(&"Virtual Machine Initialized".into());
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
                cpu_step(machine);
                machine.accumulator -= TICK_RATE;
                steps += 1;
            }

            // Render always happens once per browser frame
            // Map text mode to GPU VRAM (conceptually)
            machine.term.render(&mut machine.bus.gpu, 4, 0);
        }
    });
}

fn cpu_step(machine: &mut Machine) {
    let mut input_op = None;
    INPUT_QUEUE.with(|q| {
        if let Some(queue) = q.borrow_mut().as_mut() {
            input_op = queue.pop_front();
        }
    });
   
    // CPU Cycle
    machine.cpu.step(&mut machine.bus);
    
    machine.tick_count += 1;
    machine.frames_buffer += 1; // Count cycle for FPS
    
    match machine.state {
        MachineState::Bios => {
            if machine.bios.step(&mut machine.term, &mut machine.bus) {
                // Handoff to Kernel
                machine.state = MachineState::Kernel;
                
                // Clear BIOS Screen
                machine.term.set_bg_color(0x00_00_00_FF); // Reset to Black for Kernel
                machine.term.reset();
                machine.term.show_cursor(true); // Enable cursor for shell
                machine.bus.gpu.clear(0, 0, 0); // Clear to Black

                // Print Kernel Boot msg
                // User requested to remove welcome text and just show prompt
                machine.shell.draw_prompt(&mut machine.term);
            }
        },
        MachineState::Kernel => {
            // Process Input (Interrupts)
            if let Some(key) = input_op {
                 let res = machine.shell.on_key(&key, &mut machine.term, &mut machine.fs, machine.tick_count, machine.real_fps);
                 if res {
                     // Hard Reboot - Re-initialize everything
                     web_sys::console::log_1(&"System Rebooting...".into());
                     
                     // Hardware Init
                     let ram = hw::ram::Ram::new(16 * 1024 * 1024); 
                     let gpu = hw::gpu::Gpu::new(512, 512); 
                     let bus = hw::bus::Bus::new(ram, gpu);
                     let cpu = hw::cpu::Cpu::new();
                     let bios = bios::Bios::new();
                     
                     let term = term::Terminal::new(64, 32);
                     let shell = sys::shell::Shell::new();
                     let fs = sys::fs::FileSystem::new(10); 
                     
                     let new_machine = Machine {
                         cpu,
                         bus,
                         term,
                         shell,
                         fs,
                         tick_count: 0,
                         last_time:  web_sys::window().unwrap().performance().unwrap().now(),
                         accumulator: 0.0,
                         real_fps: 0.0,
                         frames_buffer: 0,
                         last_sec_time: web_sys::window().unwrap().performance().unwrap().now(),
                         state: MachineState::Bios,
                         bios,
                     };

                     *machine = new_machine;
                 }
            }
            machine.bus.gpu.clear(0, 0, 0); // Black background
        },
    }
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
