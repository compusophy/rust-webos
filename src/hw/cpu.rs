#![allow(dead_code)]
use super::bus::Bus;

pub struct Cpu {
    pub regs: [u32; 32], // General Purpose Registers (r0-r31)
    pub pc: u32,         // Program Counter
    pub version: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            version: 1,
        }
    }

    // Single step of the CPU
    pub fn step(&mut self, _bus: &mut Bus) { // Underscore to suppress unused for now
        // Fetch instruction (simulated)
        
        // For Phase 1: this is a dummy stepper.
        // The real "logic" is currently essentially "interrupt driven" by the Shell 
        // passing through lib.rs tick() -> cpu_step() -> shell logic.
        
        // Once we move Shell logic to run on the CPU loop, we will look at [PC]
        // and execute an instruction.
        
        self.pc += 4; // Increment PC
    }
}
