pub struct Shell {
    input_buffer: String,
    // prompt: String, // Removed, we build it dynamically
    current_path: String,
    history: Vec<String>,
    history_index: Option<usize>,
    waiting_for_reset: bool,
}

struct CommandDef {
    name: &'static str,
    desc: &'static str,
}

const COMMANDS: &[CommandDef] = &[
    CommandDef { name: "help", desc: "Show this help" },
    CommandDef { name: "clear", desc: "Clear screen" },
    CommandDef { name: "ls", desc: "List files" },
    CommandDef { name: "cd", desc: "Change directory" },
    CommandDef { name: "mkdir", desc: "Create directory" },
    CommandDef { name: "touch", desc: "Create file" },
    CommandDef { name: "df", desc: "Disk Usage" },
    CommandDef { name: "sysinfo", desc: "System Information" },
    CommandDef { name: "reboot", desc: "Reboot system" },
    CommandDef { name: "uptime", desc: "System uptime" },
    CommandDef { name: "date", desc: "Real World Time" },
    CommandDef { name: "monitor", desc: "Real System Monitor" },
    CommandDef { name: "reset", desc: "Factory Reset (Wipe Data)" },
];

impl Shell {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            current_path: "~".to_string(),
            history: Vec::new(),
            history_index: None,
            waiting_for_reset: false,
        }
    }

    pub fn draw_prompt(&self, term: &mut crate::term::Terminal) {
        // Colors
        let orange = 0xFF_A5_00_FF;
        let green = 0x00_FF_00_FF;
        let white = 0xFF_FF_FF_FF;
        
        // "user" -> Orange
        term.set_fg_color(orange);
        term.write_str("user");
        
        // "@" -> White
        term.set_fg_color(white);
        term.write_str("@");
        
        // "wasmix" -> Green
        term.set_fg_color(green);
        term.write_str("wasmix");
        
        // ":path $ " -> White
        term.set_fg_color(white);
        term.write_str(":");
        term.write_str(&self.current_path);
        term.write_str("$ ");
        
        // Reset to white for input
        term.set_fg_color(white);
    }

    pub fn update_prompt(&mut self, fs: &crate::sys::fs::FileSystem) {
        let path = if fs.current_path.is_empty() {
            "~".to_string()
        } else {
             // Join path
             let mut p = String::from("/");
             for part in &fs.current_path {
                 p.push_str(part);
                 p.push('/');
             }
             // Remove trailing slash if len > 1
             if p.len() > 1 { p.pop(); }
             p
        };
        self.current_path = path;
    }

    pub fn on_key(&mut self, key: &str, term: &mut crate::term::Terminal, fs: &mut crate::sys::fs::FileSystem, ticks: u64, hz: f64) -> bool {
        // Handle confirmation dialog
        if self.waiting_for_reset {
            if key == "Enter" {
                term.write_char('\n');
                let input = self.input_buffer.trim();
                if input == "y" || input == "Y" {
                    term.write_str("Resetting to Factory Defaults...\n");
                    
                    // Clear LocalStorage
                    if let Some(window) = web_sys::window() {
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.remove_item("wasmix_fs_local");
                        }
                    }

                    self.input_buffer.clear();
                    self.waiting_for_reset = false;
                    return true; // Trigger REBOOT
                } else {
                    term.write_str("Reset cancelled.\n");
                    self.input_buffer.clear();
                    self.waiting_for_reset = false;
                    self.draw_prompt(term);
                    return false;
                }
            } else if key == "Backspace" {
                 if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    term.write_char('\x08'); 
                }
                return false;
            } else if key.len() == 1 {
                self.input_buffer.push_str(key);
                term.write_str(key);
                return false;
            }
            return false;
        }

        if key == "Enter" {
            term.write_char('\n');
            if !self.input_buffer.trim().is_empty() {
                self.history.push(self.input_buffer.clone());
                self.history_index = None;
            }
            let reboot = self.execute_command(term, fs, ticks, hz);
            self.input_buffer.clear();
            self.draw_prompt(term);
            return reboot;
        } else if key == "Backspace" {
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                term.write_char('\x08'); 
            }
        } else if key == "ArrowUp" {
            if !self.history.is_empty() {
                let new_index = match self.history_index {
                    Some(i) => if i > 0 { i - 1 } else { 0 },
                    None => self.history.len() - 1,
                };
                self.history_index = Some(new_index);

                // Clear current line
                for _ in 0..self.input_buffer.len() {
                    term.write_char('\x08');
                }
                
                self.input_buffer = self.history[new_index].clone();
                term.write_str(&self.input_buffer);
            }
        } else if key == "ArrowDown" {
            if let Some(i) = self.history_index {
                // Clear current line
                for _ in 0..self.input_buffer.len() {
                    term.write_char('\x08');
                }

                if i < self.history.len() - 1 {
                    let new_index = i + 1;
                    self.history_index = Some(new_index);
                    self.input_buffer = self.history[new_index].clone();
                } else {
                    self.history_index = None;
                    self.input_buffer.clear();
                }
                term.write_str(&self.input_buffer);
            }
        } else if key.len() == 1 {
            self.input_buffer.push_str(key);
            term.write_str(key);
        }
        false
    }

    fn execute_command(&mut self, term: &mut crate::term::Terminal, fs: &mut crate::sys::fs::FileSystem, ticks: u64, hz: f64) -> bool {
        let cmd_str = self.input_buffer.trim();
        if cmd_str.is_empty() {
            return false;
        }

        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "help" => {
                term.write_str("Available commands:\n");
                for cmd_def in COMMANDS {
                     let msg = format!("  {:<8} - {}\n", cmd_def.name, cmd_def.desc);
                     term.write_str(&msg);
                }
            },
            "clear" => {
                term.reset();
            },
            "ls" => {
                let items = fs.list_dir();
                for item in items {
                    term.write_str(&item);
                    term.write_str("  ");
                }
                term.write_char('\n');
            },
            "mkdir" => {
                if parts.len() < 2 {
                    term.write_str("Usage: mkdir <name>\n");
                } else {
                    match fs.mkdir(parts[1]) {
                        Ok(_) => {},
                        Err(e) => {
                            term.write_str("Error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                        }
                    }
                }
            },
            "touch" => {
                if parts.len() < 2 {
                    term.write_str("Usage: touch <name>\n");
                } else {
                    match fs.create_file(parts[1]) {
                        Ok(_) => {},
                        Err(e) => {
                            term.write_str("Error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                        }
                    }
                }
            },
            "cd" => {
                if parts.len() < 2 {
                     match fs.cd("/") { _ => {} };
                } else {
                     let target = if let Some(matched) = fs.match_entry(parts[1]) {
                         matched
                     } else {
                         parts[1].to_string()
                     };
                     
                     match fs.cd(&target) {
                        Ok(_) => {},
                        Err(e) => {
                            term.write_str("Error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                        }
                    }
                }
                self.update_prompt(fs); 
            },
             "df" => {
                let total_kb = fs.total_space / 1024;
                let used_kb = fs.used_space / 1024;
                let msg = format!("Disk Usage:\n  Used: {} KB\n  Total: {} KB\n  Free: {} KB\n", used_kb, total_kb, total_kb - used_kb);
                term.write_str(&msg);
            },
            "sysinfo" => {
                term.write_str("System Information:\n");
                term.write_str("  Kernel:  Rust WebOS v0.1.0\n");
                let msg_cpu = format!("  CPU:     WASM-32 Virtual Core @ {:.2} Hz\n", hz);
                term.write_str(&msg_cpu);
                term.write_str("  Arch:    wasm32-unknown-unknown\n");
                term.write_str("  Display: 512x512 RGBA (1 MB VRAM)\n");
                term.write_str("  Memory:  16 MB Linear RAM\n");
            },
            "reboot" => {
                return true;
            },
            "uptime" => {
                let seconds = ticks as f64 / 60.0;
                let msg = format!("Uptime: {:.2} seconds ({} ticks)\n", seconds, ticks);
                term.write_str(&msg);
            },
            "date" => {
                let date = js_sys::Date::new_0();
                let msg = format!("{}\n", date.to_string());
                term.write_str(&msg);
            },
            "monitor" => {
                term.write_str("--- SYSTEM MONITOR ---\n");
                let msg_hz = format!("CPU Speed: {:.2} Hz (Target: 60 Hz)\n", hz);
                term.write_str(&msg_hz);
                term.write_str("RAM: 16 MB Linear\n");
                term.write_str("VRAM: 512x512 RGBA (1 MB)\n");
                let msg_ticks = format!("Tick Count: {}\n", ticks);
                term.write_str(&msg_ticks);
            },
            "reset" => {
                term.write_str("WARNING: This will wipe all local data.\n");
                term.write_str("Are you sure? (y/n) ");
                self.input_buffer.clear();
                self.waiting_for_reset = true;
            },
            _ => {
                term.write_str("Unknown command: ");
                term.write_str(cmd);
                term.write_char('\n');
            }
        }
        false
    }
}
