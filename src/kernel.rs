use crate::hw;
use crate::sys;
use crate::term;
use crate::bios;

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
    pub wasm: sys::wasm::WasmRuntime,
    
    // Timing and State
    pub tick_count: u64,
    pub last_time: f64,
    pub accumulator: f64,
    pub real_fps: f64,
    pub frames_buffer: u64,
    pub last_sec_time: f64,
    pub state: MachineState,
    pub gui_mode: bool,
}

impl Machine {
    pub fn new() -> Self {
        // Hardware Init
        let ram = hw::ram::Ram::new(16 * 1024 * 1024); // 16 MB RAM
        let gpu = hw::gpu::Gpu::new(512, 512); // VRAM
        let bus = hw::bus::Bus::new(ram, gpu);
        let cpu = hw::cpu::Cpu::new();
        let bios = bios::Bios::new();
        
        // Firmware/Software Init
        let term = term::Terminal::new(64, 32);
        let shell = sys::shell::Shell::new();
        let fs = sys::fs::FileSystem::new(10); // 10 MB disk
        let wasm = sys::wasm::WasmRuntime::new();
        
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
            gui_mode: false,
        }
    }

    pub fn reboot(&mut self) {
         let new_machine = Self::new();
         *self = new_machine;
    }

    pub fn step(&mut self, input_op: Option<String>) {
         // CPU Cycle
        self.cpu.step(&mut self.bus);
        
        self.tick_count += 1;
        self.frames_buffer += 1; // Count cycle for FPS
        
        match self.state {
            MachineState::Bios => {
                if self.bios.step(&mut self.term, &mut self.bus) {
                    // Handoff to Kernel
                    self.state = MachineState::Kernel;
                    
                    // Clear BIOS Screen
                    self.term.set_bg_color(0x00_00_00_FF); // Reset to Black for Kernel
                    self.term.reset();
                    self.term.show_cursor(true); // Enable cursor for shell
                    self.bus.gpu.clear(0, 0, 0); // Clear to Black
    
                    // Print Kernel Boot msg
                    self.shell.draw_prompt(&mut self.term);
                }
            },
            MachineState::Kernel => {
                // Process Input (Interrupts)
                if let Some(key) = input_op {
                     let res = self.shell.on_key(&key, &mut self.term, &mut self.fs, &self.wasm, &mut self.bus.gpu, &mut self.gui_mode, self.tick_count, self.real_fps);
                     if res {
                         // Hard Reboot - Re-initialize everything
                         web_sys::console::log_1(&"system rebooting...".into());
                         self.reboot();
                     }
                }
                
                // Clear background if not in GUI mode (handled by render loop in lib.rs actually, but let's see)
                // In lib.rs: machine.bus.gpu.clear(0, 0, 0); was at end of Kernel block.
                // It seems aggressive to clear every tick if we are also rendering terminal?
                // The original code had: machine.bus.gpu.clear(0, 0, 0); // Black background
                // This clears the VRAM buffer. Then lib.rs calls machine.term.render() which writes over it.
                // So yes, we need to clear here.
                if !self.gui_mode {
                     self.bus.gpu.clear(0, 0, 0); // Clear to Black
                }
            },
        }
    }
}
