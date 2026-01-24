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
    WaitingForInput,
    Setup,
    Booting,
    Done,
}

pub struct Bios {
    state: BiosState,
    ticks: u64,
    pub boot_target: String,
}

impl Bios {
    pub fn new() -> Self {
        Self {
            state: BiosState::PowerOn,
            ticks: 0,
            boot_target: "/bin/terminal.wasm".to_string(), // Default
        }
    }

    // Returns true when BIOS is done and Kernel should start
    pub fn step(&mut self, term: &mut Terminal, bus: &mut super::hw::bus::Bus, input_op: Option<String>) -> bool {
        self.ticks += 1;

        match self.state {
            BiosState::PowerOn => {
                bus.gpu.borrow_mut().clear(0, 0, 0); 
                
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
                     let progress = (self.ticks - 10) as f64 / 100.0;
                     let mem = (progress * 16384.0) as u32;
                     
                     if mem >= 16384 {
                         // Extra newline for spacing before "Press Any Key"
                         let msg = format!("\rmemory test: {} kb ok\n\n", 16384);
                         term.write_str(&msg);
                         // Don't print "press any key" here, let WaitingForInput handle it with countdown
                         self.state = BiosState::WaitingForInput;
                     } else {
                         let msg = format!("\rmemory test: {} kb ok", mem);
                         term.write_str(&msg);
                     }
                }
            },
            BiosState::WaitingForInput => {
                // If any key pressed, go to Setup
                if input_op.is_some() {
                    term.write_str("entering setup...\n");
                    self.state = BiosState::Setup;
                    // Reset ticks to delay setup redraw or just clear?
                    // Let's clear screen for setup menu
                    bus.gpu.borrow_mut().clear(0, 0, 0x80); // Dark Blue background for BIOS Menu
                    term.reset();
                    term.set_fg_color(0xFF_FF_00_FF); // Yellow
                    term.write_str("wasmix bios setup\n\n");
                    term.set_fg_color(0xFF_FF_FF_FF); // White
                    term.write_str("select boot device:\n");
                    term.write_str("1. terminal (default)\n");
                    term.write_str("2. desktop gui\n\n");
                    term.write_str("press [1] or [2] to select.\n");
                    return false;
                }

                // Countdown
                // WaitingForInput runs from roughly tick 110 to 250 (140 ticks).
                // Let's say timeout is tick 250.
                if self.ticks % 60 == 0 {
                     let remaining = (250i64 - self.ticks as i64) / 60;
                     if remaining > 0 {
                         let msg = format!("\rpress any key to enter setup... ({}s)   ", remaining);
                         term.write_str(&msg);
                     }
                }

                if self.ticks > 250 {
                    self.state = BiosState::Booting;
                }
            },
            BiosState::Setup => {
                if let Some(key) = input_op {
                    if key == "1" {
                        self.boot_target = "/bin/terminal.wasm".to_string();
                        term.write_str("\nselected: terminal\nbooting...");
                        self.state = BiosState::Booting;
                        self.ticks = 0; // Reset ticks for booting delay
                    } else if key == "2" {
                        self.boot_target = "/bin/desktop.wasm".to_string();
                        term.write_str("\nselected: desktop\nbooting...");
                        self.state = BiosState::Booting;
                         self.ticks = 0;
                    }
                }
            },
            BiosState::Booting => {
                 // Brief delay to read "Booting..." or "Selected..."
                 if self.ticks > 60 {
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
