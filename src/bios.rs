
use crate::term::Terminal;

pub enum BiosState {
    PowerOn,
    MemoryTest,
    Booting,
    Done,
}

pub struct Bios {
    state: BiosState,
    ticks: u64,
}

impl Bios {
    pub fn new() -> Self {
        Self {
            state: BiosState::PowerOn,
            ticks: 0,
        }
    }

    // Returns true when BIOS is done and Kernel should start
    pub fn step(&mut self, term: &mut Terminal, bus: &mut super::hw::bus::Bus) -> bool {
        self.ticks += 1;

        match self.state {
            BiosState::PowerOn => {
                bus.gpu.clear(0, 0, 50); // Blue BIOS SCreen
                
                if self.ticks == 10 {
                    term.reset();
                    term.write_str("wasmix BIOS v0.1.0\n");
                    term.write_str("Copyright (C) 2026 CompuSophy Inc.\n\n");
                    term.write_str("Detecting Hardware...\n");
                    term.write_str("CPU: WASM-32 Virtual Core\n");
                    term.write_str("Display: 512x512 RGBA (1 MB VRAM)\n");
                    self.state = BiosState::MemoryTest;
                }
            },
            BiosState::MemoryTest => {
                 if self.ticks % 10 == 0 {
                     // Check Ram (Simulate up to 16384 KB)
                     let progress = (self.ticks - 10) as f64 / 100.0; // 100 ticks for mem test
                     let mem = (progress * 16384.0) as u32;
                     
                     if mem >= 16384 {
                         let msg = format!("\rMemory Test: {} KB OK\n", 16384);
                         term.write_str(&msg);
                         self.state = BiosState::Booting;
                     } else {
                         let msg = format!("\rMemory Test: {} KB OK", mem);
                         term.write_str(&msg);
                     }
                }
            },
            BiosState::Booting => {
                if self.ticks > 150 {
                     term.write_str("\nSystem OK.\nBooting from Hard Disk...\n");
                     self.state = BiosState::Done;
                }
            },
            BiosState::Done => {
                return true;
            }
        }
        
        false
    }
}
