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

enum CmdResult {
    Success,
    Error,
    Reboot,
}

const COMMANDS: &[CommandDef] = &[
    CommandDef { name: "help", desc: "Show this help" },
    CommandDef { name: "clear", desc: "Clear screen" },
    CommandDef { name: "ls", desc: "List files" },
    CommandDef { name: "cd", desc: "Change directory" },
    CommandDef { name: "mkdir", desc: "Create directory" },
    CommandDef { name: "touch", desc: "Create file" },
    CommandDef { name: "rm", desc: "Remove file/dir" },
    CommandDef { name: "df", desc: "Disk Usage" },
    CommandDef { name: "sysinfo", desc: "System Information" },
    CommandDef { name: "reboot", desc: "Reboot system" },
    CommandDef { name: "uptime", desc: "System uptime" },
    CommandDef { name: "date", desc: "Real World Time" },
    CommandDef { name: "reset", desc: "Factory Reset (Wipe Data)" },
    CommandDef { name: "exec", desc: "Execute WASM Binary" },
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

    pub fn on_key(&mut self, key: &str, term: &mut crate::term::Terminal, fs: &mut crate::sys::fs::FileSystem, wasm: &crate::sys::wasm::WasmRuntime, gpu: &mut crate::hw::gpu::Gpu, events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> bool {
        // Handle confirmation dialog
        if self.waiting_for_reset {
            if key == "Enter" {
                term.write_char('\n');
                let input = self.input_buffer.trim();
                if input == "y" || input == "Y" {
                    term.write_str("resetting to factory defaults...\n");
                    
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
                    term.write_str("reset cancelled.\n");
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
            let reboot = self.execute_command(term, fs, wasm, gpu, events, ticks, hz);
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

    fn execute_command(&mut self, term: &mut crate::term::Terminal, fs: &mut crate::sys::fs::FileSystem, wasm: &crate::sys::wasm::WasmRuntime, gpu: &mut crate::hw::gpu::Gpu, events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> bool {
        let full_input = self.input_buffer.trim().to_string(); // Clone to break borrow
        if full_input.is_empty() {
            return false;
        }

        // Split by "&&"
        let commands: Vec<&str> = full_input.split("&&").collect();
        
        
        for cmd_str in commands {
            let res = self.run_one_command(cmd_str.trim(), term, fs, wasm, gpu, events, ticks, hz);

            match res {
                CmdResult::Success => continue,
                CmdResult::Error => break, // Stop chain on error
                CmdResult::Reboot => return true,
            }
        }
        false
    }
    
    fn run_one_command(&mut self, cmd_str: &str, term: &mut crate::term::Terminal, fs: &mut crate::sys::fs::FileSystem, wasm: &crate::sys::wasm::WasmRuntime, _gpu: &mut crate::hw::gpu::Gpu, _events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> CmdResult {
        if cmd_str.is_empty() {
             return CmdResult::Success;
        }

        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "help" => {
                term.write_str("available commands:\n");
                for cmd_def in COMMANDS {
                     let msg = format!("  {:<8} - {}\n", cmd_def.name, cmd_def.desc.to_lowercase());
                     term.write_str(&msg);
                }
                CmdResult::Success
            },
            "clear" => {
                term.reset();
                CmdResult::Success
            },
            "ls" => {
                let items = fs.list_dir();
                for item in items {
                    term.write_str(&item);
                    term.write_str("  ");
                }
                term.write_char('\n');
                CmdResult::Success
            },
            "mkdir" => {
                if parts.len() < 2 {
                    term.write_str("usage: mkdir <name>\n");
                    CmdResult::Error
                } else {
                    match fs.mkdir(parts[1]) {
                        Ok(_) => CmdResult::Success,
                        Err(e) => {
                            term.write_str("error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                            CmdResult::Error
                        }
                    }
                }
            },
            "touch" => {
                if parts.len() < 2 {
                    term.write_str("usage: touch <name>\n");
                    CmdResult::Error
                } else {
                    match fs.create_file(parts[1]) {
                        Ok(_) => CmdResult::Success,
                        Err(e) => {
                            term.write_str("error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                            CmdResult::Error
                        }
                    }
                }
            },
            "rm" => {
                if parts.len() < 2 {
                    term.write_str("usage: rm <name>\n");
                    CmdResult::Error
                } else {
                    match fs.remove_entry(parts[1]) {
                        Ok(_) => CmdResult::Success,
                        Err(e) => {
                             term.write_str("error: ");
                             term.write_str(&e);
                             term.write_char('\n');
                             CmdResult::Error
                        }
                    }
                }
            },
            "cd" => {
                if parts.len() < 2 {
                     match fs.cd("/") { _ => {} };
                     self.update_prompt(fs); 
                     CmdResult::Success
                } else {
                     let target = if let Some(matched) = fs.match_entry(parts[1]) {
                         matched
                     } else {
                         parts[1].to_string()
                     };
                     
                     match fs.cd(&target) {
                        Ok(_) => {
                            self.update_prompt(fs); 
                            CmdResult::Success
                        },
                        Err(e) => {
                            term.write_str("error: ");
                            term.write_str(&e);
                            term.write_char('\n');
                            CmdResult::Error
                        }
                    }
                }
            },
             "df" => {
                let total_kb = fs.total_space / 1024;
                let used_kb = fs.used_space / 1024;
                let msg = format!("disk usage:\n  used: {} kb\n  total: {} kb\n  free: {} kb\n", used_kb, total_kb, total_kb - used_kb);
                term.write_str(&msg);
                CmdResult::Success
            },
            "sysinfo" => {
                term.write_str("system information:\n");
                term.write_str("  kernel:  rust webos v0.1.0\n");
                term.write_str("  arch:    wasm32-unknown-unknown\n");
                let msg_cpu = format!("  cpu:     wasm-32 virtual core @ {:.2} hz\n", hz);
                term.write_str(&msg_cpu);
                term.write_str("  vram:    512x512 rgba (1 mb)\n");
                term.write_str("  ram:     16 mb linear\n");
                let msg_ticks = format!("  ticks:   {}\n", ticks);
                term.write_str(&msg_ticks);
                CmdResult::Success
            },
            "reboot" => {
                CmdResult::Reboot
            },
            "uptime" => {
                let seconds = ticks as f64 / 60.0;
                let msg = format!("uptime: {:.2} seconds ({} ticks)\n", seconds, ticks);
                term.write_str(&msg);
                CmdResult::Success
            },
            "date" => {
                let date = js_sys::Date::new_0();
                let msg = format!("{}\n", date.to_string());
                term.write_str(&msg);
                CmdResult::Success
            },

            "reset" => {
                term.write_str("warning: this will wipe all local data.\n");
                term.write_str("are you sure? (y/n) ");
                self.input_buffer.clear();
                self.waiting_for_reset = true;
                CmdResult::Success // Technically we pause here
            },
            "exec" => {
                if parts.len() < 2 {
                    term.write_str("usage: exec <path>\n");
                    CmdResult::Error
                } else {
                    let path = parts[1];
                    // Read file content
                    let file_node = fs.resolve_dir(&fs.current_path) // Start from current dir defaults
                        .and_then(|d| d.children.get(path))
                        .or_else(|| {
                             // Try absolute path resolution (simple /bin/hello.wasm check)
                             if path.starts_with("/bin/") {
                                 fs.root.children.get("bin").and_then(|b| b.children.get(&path[5..]))
                             } else {
                                None
                             }
                        });


                    if let Some(node) = file_node {
                        if let crate::sys::fs::NodeType::File = node.node_type {
                            term.write_str(&format!("executing {}...\n", path));
                            match wasm.load(&node.content) {
                                Ok(_) => {
                                    // Loaded successfully. active_process is set.
                                    CmdResult::Success
                                },
                                Err(e) => {
                                    term.write_str(&format!("execution error: {}\n", e));
                                    CmdResult::Error
                                }
                            }
                        } else {
                             term.write_str("not a file\n");
                             CmdResult::Error
                        }
                    } else {
                        // Fallback: Try resolving properly if we can or just hack it for "hello.wasm"
                         let dir = fs.resolve_dir(&fs.current_path).unwrap_or(&fs.root);
                         if let Some(node) = dir.children.get(path) {
                               if let crate::sys::fs::NodeType::File = node.node_type {
                                    match wasm.load(&node.content) {
                                        Ok(_) => CmdResult::Success,
                                        Err(e) => {
                                             term.write_str(&format!("error: {}\n", e));
                                             CmdResult::Error
                                        }
                                    }
                               } else {
                                   term.write_str("not a file\n");
                                   CmdResult::Error
                               }
                         } else {
                             term.write_str("file not found\n");
                             CmdResult::Error
                         }
                    }
                }
            },
            _ => {
                term.write_str("unknown command: ");
                term.write_str(cmd);
                term.write_char('\n');
                CmdResult::Error
            }
        }
    }
}
