

pub struct Shell {
    input_buffer: String,
    current_path: String,
    history: Vec<String>,
    history_index: Option<usize>,
    waiting_for_reset: bool,
}

struct CommandDef {
    name: &'static str,
    desc: &'static str,
}

#[derive(PartialEq)]
pub enum CmdResult {
    Success,
    Error,
    Reboot,
    Clear,
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

    pub fn draw_prompt(&self, term: &std::rc::Rc<std::cell::RefCell<crate::term::Terminal>>) {
        let mut term = term.borrow_mut();
        
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

    pub fn update_prompt(&mut self, fs: &std::rc::Rc<std::cell::RefCell<crate::sys::fs::FileSystem>>) {
        let path = {
            let fs_guard = fs.borrow();
            if fs_guard.current_path.is_empty() {
                "~".to_string()
            } else {
                 // Join path
                 let mut p = String::from("/");
                 for part in &fs_guard.current_path {
                     p.push_str(part);
                     p.push('/');
                 }
                 // Remove trailing slash if len > 1
                 if p.len() > 1 { p.pop(); }
                 p
            }
        };
        self.current_path = path;
    }

    pub fn on_key(&mut self, key: &str, term: &std::rc::Rc<std::cell::RefCell<crate::term::Terminal>>, fs: &std::rc::Rc<std::cell::RefCell<crate::sys::fs::FileSystem>>, wasm: &crate::sys::wasm::WasmRuntime, events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> bool {
        // ... (lines 98-135 unchanged logical structure, but prompt update needs fs)
        // Handle confirmation dialog
        if self.waiting_for_reset {
            if key == "Enter" {
                term.borrow_mut().write_char('\n');
                let input = self.input_buffer.trim();
                if input == "y" || input == "Y" {
                    term.borrow_mut().write_str("resetting to factory defaults...\n");
                    
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
                    term.borrow_mut().write_str("reset cancelled.\n");
                    self.input_buffer.clear();
                    self.waiting_for_reset = false;
                    self.draw_prompt(term);
                    return false;
                }
            } else if key == "Backspace" {
                 if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    term.borrow_mut().write_char('\x08'); 
                }
                return false;
            } else if key.len() == 1 {
                self.input_buffer.push_str(key);
                term.borrow_mut().write_str(key);
                return false;
            }
            return false;
        }

        if key == "Enter" {
            term.borrow_mut().write_char('\n');
            if !self.input_buffer.trim().is_empty() {
                self.history.push(self.input_buffer.clone());
                self.history_index = None;
            }
            
            let reboot = self.execute_command(term, fs, Some(wasm), events, ticks, hz);
            self.input_buffer.clear();
            self.draw_prompt(term);
            return reboot;
        } else if key == "Backspace" {
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                term.borrow_mut().write_char('\x08'); 
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
                    term.borrow_mut().write_char('\x08');
                }
                
                self.input_buffer = self.history[new_index].clone();
                term.borrow_mut().write_str(&self.input_buffer);
            }
        } else if key == "ArrowDown" {
            if let Some(i) = self.history_index {
                // Clear current line
                for _ in 0..self.input_buffer.len() {
                    term.borrow_mut().write_char('\x08');
                }

                if i < self.history.len() - 1 {
                    let new_index = i + 1;
                    self.history_index = Some(new_index);
                    self.input_buffer = self.history[new_index].clone();
                } else {
                    self.history_index = None;
                    self.input_buffer.clear();
                }
                term.borrow_mut().write_str(&self.input_buffer);
            }
        } else if key.len() == 1 {
            self.input_buffer.push_str(key);
            term.borrow_mut().write_str(key);
        }
        false
    }
    
    pub fn execute_command(&mut self, term: &std::rc::Rc<std::cell::RefCell<crate::term::Terminal>>, fs: &std::rc::Rc<std::cell::RefCell<crate::sys::fs::FileSystem>>, wasm: Option<&crate::sys::wasm::WasmRuntime>, events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> bool {
        let full_input = self.input_buffer.trim().to_string(); 
        if full_input.is_empty() {
             return false;
        }

        let commands: Vec<&str> = full_input.split("&&").collect();
        
        for cmd_str in commands {
            let (res, output) = self.run_one_command(cmd_str.trim(), fs, wasm, events, ticks, hz);
            
            term.borrow_mut().write_str(&output);

            match res {
                CmdResult::Success => continue,
                CmdResult::Error => break, 
                CmdResult::Reboot => return true,
                CmdResult::Clear => term.borrow_mut().reset(),
            }
        }
        false
    }
    
    pub fn execute_string(&mut self, full_input: &str, fs: &std::rc::Rc<std::cell::RefCell<crate::sys::fs::FileSystem>>, wasm: Option<&crate::sys::wasm::WasmRuntime>, events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> String {
        let mut full_output = String::new();
        if full_input.trim().is_empty() {
            return full_output;
        }
        
        let commands: Vec<&str> = full_input.split("&&").collect();
        
        for cmd_str in commands {
            // Note: pass dummy term? Or does execute_string NOT print to term?
            // Actually, run_one_command returns output. We aggregate it.
            // But we can't pass `term` because we don't have it here. This function returns String.
            // That's fine.
            let (res, output) = self.run_one_command(cmd_str.trim(), fs, wasm, events, ticks, hz);
            full_output.push_str(&output);

            match res {
                CmdResult::Success => continue,
                CmdResult::Error => break, 
                CmdResult::Reboot => full_output.push_str("System Rebooting...\n"),
                CmdResult::Clear => {}, 
            }
        }
        full_output
    }
    
    fn run_one_command(&mut self, cmd_str: &str, fs: &std::rc::Rc<std::cell::RefCell<crate::sys::fs::FileSystem>>, wasm: Option<&crate::sys::wasm::WasmRuntime>, _events: &mut std::collections::VecDeque<crate::kernel::SystemEvent>, ticks: u64, hz: f64) -> (CmdResult, String) {
        let mut out = String::new();
        if cmd_str.is_empty() {
             return (CmdResult::Success, out);
        }

        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "help" => {
                out.push_str("available commands:\n");
                for cmd_def in COMMANDS {
                     let msg = format!("  {:<8} - {}\n", cmd_def.name, cmd_def.desc.to_lowercase());
                     out.push_str(&msg);
                }
                (CmdResult::Success, out)
            },
            "clear" => {
                (CmdResult::Clear, out)
            },
            "ls" => {
                let items = fs.borrow().list_dir();
                for item in items {
                    out.push_str(&item);
                    out.push_str("  ");
                }
                out.push('\n');
                (CmdResult::Success, out)
            },
            "mkdir" => {
                if parts.len() < 2 {
                    out.push_str("usage: mkdir <name>\n");
                    (CmdResult::Error, out)
                } else {
                    let res = fs.borrow_mut().mkdir(parts[1]);
                    match res {
                        Ok(_) => (CmdResult::Success, out),
                        Err(e) => {
                            out.push_str("error: ");
                            out.push_str(&e);
                            out.push('\n');
                            (CmdResult::Error, out)
                        }
                    }
                }
            },
            "touch" => {
                if parts.len() < 2 {
                    out.push_str("usage: touch <name>\n");
                    (CmdResult::Error, out)
                } else {
                    let res = fs.borrow_mut().create_file(parts[1]);
                    match res {
                        Ok(_) => (CmdResult::Success, out),
                        Err(e) => {
                            out.push_str("error: ");
                            out.push_str(&e);
                            out.push('\n');
                            (CmdResult::Error, out)
                        }
                    }
                }
            },
            "rm" => {
                if parts.len() < 2 {
                    out.push_str("usage: rm <name>\n");
                    (CmdResult::Error, out)
                } else {
                    let res = fs.borrow_mut().remove_entry(parts[1]);
                    match res {
                        Ok(_) => (CmdResult::Success, out),
                        Err(e) => {
                             out.push_str("error: ");
                             out.push_str(&e);
                             out.push('\n');
                             (CmdResult::Error, out)
                        }
                    }
                }
            },
            "cd" => {
                if parts.len() < 2 {
                     let _ = fs.borrow_mut().cd("/");
                     self.update_prompt(fs); 
                     (CmdResult::Success, out)
                } else {
                     let target = if let Some(matched) = fs.borrow().match_entry(parts[1]) {
                         matched
                     } else {
                         parts[1].to_string()
                     };
                     
                     let res = fs.borrow_mut().cd(&target);
                     match res {
                        Ok(_) => {
                            self.update_prompt(fs); 
                            (CmdResult::Success, out)
                        },
                        Err(e) => {
                            out.push_str("error: ");
                            out.push_str(&e);
                            out.push('\n');
                            (CmdResult::Error, out)
                        }
                    }
                }
            },
             "df" => {
                let fs_guard = fs.borrow();
                let total_kb = fs_guard.total_space / 1024;
                let used_kb = fs_guard.used_space / 1024;
                let msg = format!("disk usage:\n  used: {} kb\n  total: {} kb\n  free: {} kb\n", used_kb, total_kb, total_kb - used_kb);
                out.push_str(&msg);
                (CmdResult::Success, out)
            },
            "sysinfo" => {
                out.push_str("system information:\n");
                out.push_str("  kernel:  rust webos v0.1.0\n");
                out.push_str("  arch:    wasm32-unknown-unknown\n");
                let msg_cpu = format!("  cpu:     wasm-32 virtual core @ {:.2} hz\n", hz);
                out.push_str(&msg_cpu);
                out.push_str("  vram:    512x512 rgba (1 mb)\n");
                out.push_str("  ram:     16 mb linear\n");
                let msg_ticks = format!("  ticks:   {}\n", ticks);
                out.push_str(&msg_ticks);
                (CmdResult::Success, out)
            },
            "reboot" => {
                (CmdResult::Reboot, out)
            },
            "uptime" => {
                let seconds = ticks as f64 / 60.0;
                let msg = format!("uptime: {:.2} seconds ({} ticks)\n", seconds, ticks);
                out.push_str(&msg);
                (CmdResult::Success, out)
            },
            "date" => {
                let date = js_sys::Date::new_0();
                let msg = format!("{}\n", date.to_string());
                out.push_str(&msg);
                (CmdResult::Success, out)
            },

            "reset" => {
                out.push_str("warning: this will wipe all local data.\n");
                out.push_str("are you sure? (y/n) ");
                self.input_buffer.clear();
                self.waiting_for_reset = true;
                (CmdResult::Success, out) 
            },
            "exec" => {
                if parts.len() < 2 {
                    out.push_str("usage: exec <path>\n");
                    (CmdResult::Error, out)
                } else if wasm.is_none() {
                     out.push_str("exec not supported in this environment\n");
                     (CmdResult::Error, out)
                } else {
                    let wasm_rt = wasm.unwrap();
                    let path = parts[1];
                    
                    // Critical: DO NOT hold FS lock here. wasm_rt.load_from_path will take it.
                    let file_node_content = {
                        let fs_guard = fs.borrow();
                        fs_guard.resolve_dir(&fs_guard.current_path) 
                            .and_then(|d| d.children.get(path))
                            .or_else(|| {
                                 if path.starts_with("/bin/") {
                                     fs_guard.root.children.get("bin").and_then(|b| b.children.get(&path[5..]))
                                 } else {
                                    None
                                 }
                            }).and_then(|node| {
                                if let crate::sys::fs::NodeType::File = node.node_type {
                                    Some(node.content.clone()) // Clone content to ensure we drop fs borrow
                                } else {
                                    None
                                }
                            })
                    };

                    if let Some(content) = file_node_content {
                        out.push_str(&format!("loading wasm ({} bytes)...\n", content.len()));
                        match wasm_rt.load(&content) {
                            Ok(captured_output) => {
                                out.push_str(&captured_output);
                                (CmdResult::Success, out)
                            },
                            Err(e) => {
                                out.push_str(&format!("exec crash: {}\n", e));
                                (CmdResult::Error, out)
                            }
                        }
                    } else {
                         // Check if directory?
                         let is_dir = {
                             let fs_guard = fs.borrow();
                             // Debug path resolution
                             // web_sys::console::log_1(&format!("exec resolve failed for: {}", path).into());
                             let dir = fs_guard.resolve_dir(&fs_guard.current_path).unwrap_or(&fs_guard.root);
                             dir.children.get(path).is_some()
                         };
                         
                         if is_dir {
                             out.push_str(&format!("error: '{}' is a directory\n", path));
                             (CmdResult::Error, out)
                         } else {
                             out.push_str(&format!("error: file '{}' not found\n", path));
                             (CmdResult::Error, out)
                         }
                    }
                }
            },
            _ => {
                out.push_str("unknown command: ");
                out.push_str(cmd);
                out.push('\n');
                (CmdResult::Error, out)
            }
        }
    }
}
