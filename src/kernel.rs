use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::hw;
use crate::sys;
use crate::term;
use crate::bios;

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    KeyDown = 1,
    KeyUp = 2,
    MouseDown = 3,
    MouseUp = 4,
    MouseMove = 5,
}

#[derive(Clone, Copy, Debug)]
pub struct SystemEvent {
    pub event_type: EventType,
    pub code: u32, // KeyCode or Button
    pub x: i32,
    pub y: i32,
}

pub enum MachineState {
    Bios,
    Kernel,
}

pub struct Machine {
    pub cpu: hw::cpu::Cpu,
    pub bus: hw::bus::Bus, // Holds Rcs
    pub bios: bios::Bios,
    
    // Peripherals / Firmware
    pub term: Rc<RefCell<term::Terminal>>,
    pub shell: Rc<RefCell<sys::shell::Shell>>,
    pub fs: Rc<RefCell<sys::fs::FileSystem>>,
    pub wasm: sys::wasm::WasmRuntime,
    
    // Timing and State
    pub tick_count: u64,
    pub last_time: f64,
    pub accumulator: f64,
    pub real_fps: f64,
    pub frames_buffer: u64,
    pub last_sec_time: f64,
    pub state: MachineState,
    pub gui_mode: Rc<RefCell<bool>>,
    pub should_reset: Rc<RefCell<bool>>,
    
    // Input
    pub events: Rc<RefCell<VecDeque<SystemEvent>>>,
}

impl Machine {
    pub fn new() -> Self {
        // Hardware Init
        let ram = Rc::new(RefCell::new(hw::ram::Ram::new(16 * 1024 * 1024))); // 16 MB RAM
        let gpu = Rc::new(RefCell::new(hw::gpu::Gpu::new(512, 512))); // VRAM
        let bus = hw::bus::Bus::new(ram.clone(), gpu.clone());
        let cpu = hw::cpu::Cpu::new();
        let bios = bios::Bios::new();
        
        // Firmware/Software Init
        let term = Rc::new(RefCell::new(term::Terminal::new(64, 32)));
        let shell = Rc::new(RefCell::new(sys::shell::Shell::new()));
        let fs = Rc::new(RefCell::new(sys::fs::FileSystem::new(10))); // 10 MB disk
        
        // Shared State
        let gui_mode = Rc::new(RefCell::new(false));
         let events = Rc::new(RefCell::new(VecDeque::new()));
         let should_reset = Rc::new(RefCell::new(false));

        // Wasm Runtime needs access to these Rcs
        let wasm = sys::wasm::WasmRuntime::new(
            term.clone(),
            gpu.clone(),
            gui_mode.clone(),
            events.clone(),
            fs.clone(),
            should_reset.clone(),
            shell.clone()
        );
        
        let now = web_sys::window().unwrap().performance().unwrap().now();

        Self {
            cpu,
            bus,
            term,
            shell,
            fs,
            wasm,
            tick_count: 0,
            last_time: now,
            accumulator: 0.0,
            real_fps: 0.0,
            frames_buffer: 0,
            last_sec_time: now,
            state: MachineState::Bios,
            bios,
            gui_mode,
            should_reset,
            events,
        }
    }

    pub fn reboot(&mut self) {
         let new_machine = Self::new();
         *self = new_machine;
    }

    pub fn tick_process(&mut self) {
        self.wasm.tick();
    }

    pub fn step(&mut self, _input_op: Option<String>) {
         // CPU Cycle
        self.cpu.step(&mut self.bus);
        
        self.tick_count += 1;
        self.frames_buffer += 1; // Count cycle for FPS
        
        match self.state {
            MachineState::Bios => {
                // Borrow check: bios needs mutable access to term and bus.
                // We have Rcs. 
                // term is Rc<RefCell<>>.
                // bus has Rcs inside.
                // self.bios.step signature: (&mut Terminal, &mut Bus)
                // We need to borrow_mut() term.
                // We need to pass bus.
                
                let mut term = self.term.borrow_mut();
                if self.bios.step(&mut term, &mut self.bus, _input_op) {
                    // Handoff to Kernel
                    self.state = MachineState::Kernel;
                    
                    // Clear Input Events accumulated during BIOS (prevent double input in Terminal)
                    self.events.borrow_mut().clear();
                    
                    // Clear BIOS Screen
                    term.set_bg_color(0x00_00_00_FF); 
                    term.reset();
                    term.show_cursor(true); 
                    self.bus.gpu.borrow_mut().clear(0, 0, 0); 
    
                    // NEW BOOT LOGIC:
                    // Load selected boot target from BIOS
                    let target = &self.bios.boot_target;
                    
                    if let Err(e) = self.wasm.load_from_path(target) {
                        web_sys::console::log_1(&format!("Failed to boot {}: {}", target, e).into());
                        // Fallback?
                        term.write_str(&format!("boot error: {}\n", e));
                    }
                    
                    // If we booted Terminal (or Desktop which uses Term for logs), we're good.
                    // Desktop might switch to GUI Mode by itself?
                    // `desktop.wasm` calls `sys_enable_gui_mode` in its init.
                }
            },
            MachineState::Kernel => {
                 // Check Shared Reset Flag (From WASM)
                 if *self.should_reset.borrow() {
                     self.reboot();
                     return;
                 }
                 
                // Process Input (Interrupts) - NOW handled by Machine.events + Shell on_key?
                // `step` received `input_op` (legacy from lib.rs InputQueue).
                // We should stop using input_op and look at `events`.
                // Actually Shell expects strings for keys.
                // Let's pop from events if it's a KeyDown? Use `events` directly?
                // Shell needs to consume events.
                
                let events_guard = self.events.borrow_mut();
                if let Some(event) = events_guard.front().cloned() {
                    // Check if it is a key event to pass to shell?
                    // Or does shell process the queue? 
                    // Shell `on_key` takes a single key.
                    // If we have multiple events, we process one per tick?
                    
                    // If WASM is active (Desktop), it consumes events via syscalls.
                    // If WASM is NOT active (Shell), Shell consumes events?
                    // Currently `wasm` is always active in the struct, but maybe no process loaded?
                    // `wasm.is_running()`?
                    
                    // If Desktop is running, Shell should arguably yield?
                    // But our Shell is the OS. 
                    // Let's say: If `gui_mode` is TRUE, Shell ignores input (Desktop handles).
                    // If `gui_mode` is FALSE, Shell handles input.
                    
                    let gui = *self.gui_mode.borrow();
                    if gui {
                        // Desktop Mode: WASM handles events via polling
                        // Do nothing here, wasm.tick() will happen.
                    } else {
                        // Text Mode: Shell handles input
                         match event.event_type {
                            EventType::KeyDown => {
                                // We need to convert code to string. 
                                // Since we didn't implement real key codes yet (passed 0 in lib.rs),
                                // we rely on the fact that we passed NOTHING in lib.rs?
                                // Wait, lib.rs::on_keydown was modified to push to `events`.
                                // BUT it also pushed to `INPUT_QUEUE`.
                                // Let's keep using `INPUT_QUEUE` for Shell for now to avoid breaking Shell?
                                // No, I want to unify.
                                // I'll stick to legacy `input_op` passed from `tick` for SHELL commands?
                                // OK, `input_op` is the `INPUT_QUEUE`.
                            },
                             _ => {}
                         }
                    }
                }
                drop(events_guard); // Release borrow
                
                // Legacy Shell Input (Text Mode) - Can be removed if we fully deprecate internal shell
                if !*self.gui_mode.borrow() {
                     if let Some(key) = _input_op {
                         let should_reboot = {
                             // let mut term = self.term.borrow_mut(); // REMOVED: Shell handles locking
                             let term = self.term.clone();
                             // Pass Rc directly (Shell manages locks)
                             let fs = self.fs.clone(); 
                             // let mut gpu = self.bus.gpu.borrow_mut(); // Removed to avoid RefCell panic
                             let mut shell = self.shell.borrow_mut();
                             
                             let mut events = self.events.borrow_mut();
                             
                             shell.on_key(&key, &term, &fs, &self.wasm, &mut events, self.tick_count, self.real_fps)
                         };
                         
                         if should_reboot {
                             web_sys::console::log_1(&"system rebooting...".into());
                             self.reboot(); 
                         }
                     }
                }

                // REMOVED CLEAR: Terminal App manages screen.
            },
        }
    }
}
