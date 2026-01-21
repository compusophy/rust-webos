pub struct Shell {
    input_buffer: String,
    prompt: String,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            prompt: "user@webos:~$ ".to_string(),
        }
    }

    pub fn draw_prompt(&self, term: &mut crate::term::Terminal) {
        term.write_str(&self.prompt);
    }

    pub fn update_prompt(&mut self, fs: &crate::fs::FileSystem) {
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
        self.prompt = format!("user@webos:{} $ ", path);
    }

    pub fn on_key(&mut self, key: &str, term: &mut crate::term::Terminal, fs: &mut crate::fs::FileSystem) -> bool {
        if key == "Enter" {
            term.write_char('\n');
            let reboot = self.execute_command(term, fs);
            self.input_buffer.clear();
            self.draw_prompt(term);
            return reboot;
        } else if key == "Backspace" {
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
                term.write_char('\x08'); 
            }
        } else if key.len() == 1 {
            self.input_buffer.push_str(key);
            term.write_str(key);
        }
        false
    }

    fn execute_command(&mut self, term: &mut crate::term::Terminal, fs: &mut crate::fs::FileSystem) -> bool {
        let cmd_str = self.input_buffer.trim();
        if cmd_str.is_empty() {
            return false;
        }

        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "help" => {
                term.write_str("Available commands:\n");
                term.write_str("  help     - Show this help\n");
                term.write_str("  clear    - Clear screen\n");
                term.write_str("  ls       - List files\n");
                term.write_str("  cd <dir> - Change directory\n");
                term.write_str("  mkdir    - Create directory\n");
                term.write_str("  df       - Disk Usage\n");
                term.write_str("  sysinfo  - System Information\n");
                term.write_str("  reboot   - Reboot system\n");
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
            "cd" => {
                if parts.len() < 2 {
                     match fs.cd("/") { _ => {} };
                } else {
                     match fs.cd(parts[1]) {
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
                term.write_str("  Arch:    wasm32-unknown-unknown\n");
                term.write_str("  Display: 512x512 RGBA (1 MB VRAM)\n");
                term.write_str("  Memory:  Dynamic WASM Linear Memory\n");
            },
            "reboot" => {
                return true;
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
