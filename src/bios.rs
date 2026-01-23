/*
 * MIT License
 * 
 * Copyright (c) 2026 CompuSophy
 * 
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * 
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 */

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
                bus.gpu.clear(0, 0, 0); // Black BIOS SCreen
                
                if self.ticks == 10 {
                    term.reset();
                    term.write_str("wasmix bios v0.1.0\n");
                    term.write_str("copyright (c) 2026 compusophy inc.\n\n");
                    term.write_str("detecting hardware...\n");
                    term.write_str("cpu: wasm-32 virtual core\n");
                    term.write_str("display: 512x512 rgba (1 mb vram)\n");
                    self.state = BiosState::MemoryTest;
                }
            },
            BiosState::MemoryTest => {
                 if self.ticks % 10 == 0 {
                     // Check Ram (Simulate up to 16384 KB)
                     let progress = (self.ticks - 10) as f64 / 100.0; // 100 ticks for mem test
                     let mem = (progress * 16384.0) as u32;
                     
                     if mem >= 16384 {
                         let msg = format!("\rmemory test: {} kb ok\n", 16384);
                         term.write_str(&msg);
                         self.state = BiosState::Booting;
                     } else {
                         let msg = format!("\rmemory test: {} kb ok", mem);
                         term.write_str(&msg);
                     }
                }
            },
            BiosState::Booting => {
                 // Skip text, just wait a brief moment then done
                 if self.ticks > 120 {
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
