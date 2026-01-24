use crate::term::Terminal;
use crate::ui;

pub struct Shell {
    pub term: Terminal,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub awaiting_restart_confirm: bool,
}

impl Shell {
    pub fn new(cols: usize, rows: usize) -> Self {
        let term = Terminal::new(cols, rows);
        
        // Initial Prompt
        let mut shell = Self {
            term,
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: None,
            awaiting_restart_confirm: false,
        };
        
        shell.write_prompt();
        shell
    }

    pub fn write_prompt(&mut self) {
        // "user" -> Orange
        self.term.set_fg_color(0xFF_A5_00_FFu32 as i32);
        self.term.write_str("user");
        
        // "@" -> White
        self.term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
        self.term.write_str("@");
        
        // "wasmix" -> Green
        self.term.set_fg_color(0x00_FF_00_FFu32 as i32);
        self.term.write_str("wasmix");
        
        // ":path $ " -> White
        let path = ui::getcwd();
        self.term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
        self.term.write_str(":");
        self.term.write_str(&path);
        self.term.write_str("$ ");
    }

    pub fn on_key(&mut self, code: u32) {
        if self.awaiting_restart_confirm {
            if code == 10 { // Enter
                self.term.write_char('\n');
                let input = self.input_buffer.trim();
                if input == "y" || input == "Y" {
                    // Call sys_restart
                    unsafe { ui::sys_restart(); }
                } else {
                     self.term.write_str("restart cancelled.\n");
                }
                self.input_buffer.clear();
                self.awaiting_restart_confirm = false;
                self.write_prompt();
            } else if code == 8 { // Backspace
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    self.term.write_char('\x08');
                }
            } else {
                 if let Some(c) = std::char::from_u32(code) {
                    if c.is_ascii_graphic() || c == ' ' {
                         self.input_buffer.push(c);
                         self.term.write_char(c);
                    }
                 }
            }
            return;
        }

        match code {
            10 => { // Enter
                let cmd = self.input_buffer.trim().to_string();
                
                // New Line
                self.term.write_char('\n'); 
                
                if !cmd.is_empty() {
                    self.history.push(cmd.clone());
                    self.history_index = None;
                    
                    if cmd == "clear" {
                        self.term.reset();
                        self.write_prompt();
                        self.input_buffer.clear();
                        return;
                    } else if cmd == "restart" {
                        self.term.write_str("confirm restart? (y/n) ");
                        self.input_buffer.clear();
                        self.awaiting_restart_confirm = true;
                        return;
                    } else {
                        // Exec
                        let output = ui::exec(&cmd);
                        
                        if !output.is_empty() {
                            // Output is White
                            self.term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
                            self.term.write_str(&output);
                            if !output.ends_with('\n') {
                                self.term.write_char('\n');
                            }
                        }
                    }
                }
                
                self.input_buffer.clear();
                self.write_prompt();
            },
            8 => { // Backspace
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    self.term.write_char('\x08');
                }
            },
            38 => { // Up Arrow
                if !self.history.is_empty() {
                     let idx = match self.history_index {
                         Some(i) => if i > 0 { i - 1 } else { 0 },
                         None => self.history.len() - 1,
                     };
                     self.history_index = Some(idx);
                     // Clear current input line visual
                     self.clear_input_line();
                     self.input_buffer = self.history[idx].clone();
                     self.term.write_str(&self.input_buffer);
                }
            },
            40 => { // Down Arrow
                 if let Some(idx) = self.history_index {
                     if idx < self.history.len() - 1 {
                         let new_idx = idx + 1;
                         self.history_index = Some(new_idx);
                         self.clear_input_line();
                         self.input_buffer = self.history[new_idx].clone();
                         self.term.write_str(&self.input_buffer);
                     } else {
                         // Back to empty
                         self.history_index = None;
                         self.clear_input_line();
                         self.input_buffer.clear();
                     }
                 }
            },
            _ => {
                if let Some(c) = std::char::from_u32(code) {
                    if c.is_ascii_graphic() || c == ' ' {
                        self.input_buffer.push(c);
                        // Input stays Colored (Green)
                        self.term.set_fg_color(0x00_FF_00_FFu32 as i32); 
                        self.term.write_char(c);
                    }
                }
            }
        }
    }
    
    fn clear_input_line(&mut self) {
        // Rudimentary line clearing: backspace len times?
        // Or specific ANSI-like trigger?
        // Since we are drawing to buffer, we can just "backspace" N times
        let len = self.input_buffer.len();
        for _ in 0..len {
            self.term.write_char('\x08');
        }
    }

    pub fn draw(&self, offset_x: i32, offset_y: i32) {
        self.term.render(offset_x, offset_y);
    }
}
