use crate::ui;

pub struct Window {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub title: String,
    pub content_type: String, // "file_manager", "terminal"
    pub is_dragging: bool,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    pub minimized: bool, // NEW
    
    // State for File Manager
    pub current_path: String,
    pub files: Vec<(bool, String)>,
    
    // State for Terminal
    pub term_lines: Vec<String>,
    pub term_input: String,
}

impl Window {
    pub fn new(id: usize, x: i32, y: i32, w: i32, h: i32, title: &str, content_type: &str) -> Self {
        let mut win = Self {
            id,
            x, y, w, h,
            title: title.to_string(),
            content_type: content_type.to_string(),
            is_dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
            minimized: false, // NEW
            current_path: "/".to_string(),
            files: Vec::new(),
            term_lines: Vec::new(),
            term_input: String::new(),
        };
        
        if content_type == "file_manager" {
            win.refresh_files();
        } else if content_type == "terminal" {
            // No welcome message to match Kernel Shell
        }
        
        win
    }
    
    pub fn on_key(&mut self, code: u32) {
        if self.minimized { return; } // NEW: Ignore input if minimized
        if self.content_type != "terminal" { return; }
        
        match code {
            10 => { // Enter
                let cmd = self.term_input.trim().to_string();
                self.term_lines.push(format!("user@wasmix:~$ {}", self.term_input));
                self.term_input.clear();
                
                // Parse Command
                if cmd == "clear" {
                     self.term_lines.clear();
                } else if !cmd.is_empty() {
                     let output = ui::exec(&cmd);
                     for line in output.lines() {
                         self.term_lines.push(line.to_string());
                     }
                }
                
                // Keep history limited
                if self.term_lines.len() > 14 {
                    let remove = self.term_lines.len() - 14;
                    self.term_lines.drain(0..remove);
                }
            },
            8 => { // Backspace
                self.term_input.pop();
            },
            _ => {
                if let Some(c) = std::char::from_u32(code) {
                     if c.is_ascii_graphic() || c == ' ' {
                         self.term_input.push(c);
                     }
                }
            }
        }
    }
    
    pub fn refresh_files(&mut self) {
        self.files = ui::read_dir(&self.current_path);
    }
    
    pub fn contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn title_bar_contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        // Title bar is 22px height approx (including borders)
        // From drawing code: y..y+25 area generally
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + 25
    }

    pub fn close_button_contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        // Close button: right aligned.
        let btn_x = self.x + self.w - 20;
        let btn_y = self.y + 5;
        let btn_w = 14;
        let btn_h = 14;
        x >= btn_x && x < btn_x + btn_w && y >= btn_y && y < btn_y + btn_h
    }
    
    pub fn minimize_button_contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        // Minimize button: left of close button.
        // Close is at w-20. Minimize at w-38? (14px width + 4px gap)
        let btn_x = self.x + self.w - 38;
        let btn_y = self.y + 5;
        let btn_w = 14;
        let btn_h = 14;
        x >= btn_x && x < btn_x + btn_w && y >= btn_y && y < btn_y + btn_h
    }

    // UPDATED: Accept optional reference to all windows for Task Manager
    pub fn draw(&self, is_active: bool, windows: Option<&Vec<Window>>) {
        if self.minimized { return; } // Don't draw if minimized
        
        unsafe {
             // Shadow
            ui::sys_draw_rect(self.x + 5, self.y + 5, self.w, self.h, 0x00_00_00_80); 
            
            // Frame
            ui::draw_panel_raised(self.x, self.y, self.w, self.h);

            // Title Bar
            let title_color = if is_active { ui::COLOR_BLUE } else { ui::COLOR_GRAY };
            ui::sys_draw_rect(self.x + 3, self.y + 3, self.w - 6, 18, title_color);
            
            // Title Text
            ui::draw_text(self.x + 6, self.y + 6, &self.title, ui::COLOR_WHITE);
            
            // Close Button
            let close_x = self.x + self.w - 20;
            let btn_y = self.y + 5;
            ui::draw_panel_raised(close_x, btn_y, 14, 14);
            ui::draw_text(close_x + 4, btn_y + 1, "x", ui::COLOR_WHITE);

            // Minimize Button
            let min_x = self.x + self.w - 38;
            ui::draw_panel_raised(min_x, btn_y, 14, 14);
            ui::draw_text(min_x + 4, btn_y + 1, "_", ui::COLOR_WHITE); 

            // Content Area
            let content_x = self.x + 4;
            let content_y = self.y + 24;
            let content_w = self.w - 8;
            let content_h = self.h - 28;
            
            // White bg for content usually
            let bg = if self.content_type == "terminal" { ui::COLOR_BLACK } else { ui::COLOR_GRAY }; 
            ui::draw_panel_sunken(content_x, content_y, content_w, content_h, bg);
            
            // Draw Context
            match self.content_type.as_str() {
                "file_manager" => {
                    // Header
                    ui::draw_text(content_x + 5, content_y + 5, &format!("path: {}", self.current_path), ui::COLOR_WHITE);
                    ui::sys_draw_rect(content_x + 5, content_y + 16, content_w - 10, 1, ui::COLOR_DARK_GRAY);
                    
                    let mut dy = content_y + 20;
                    for (is_dir, name) in &self.files {
                         let icon = if *is_dir { "[dir] " } else { "      " };
                         ui::draw_text(content_x + 5, dy, icon, ui::COLOR_WHITE);
                         ui::draw_text(content_x + 40, dy, name, ui::COLOR_WHITE);
                         dy += 16;
                    }
                },
                "terminal" => {
                     // Draw Lines
                     let mut dy = content_y + 5;
                     for line in &self.term_lines {
                         ui::draw_text(content_x + 5, dy, line, ui::COLOR_WHITE);
                         dy += 16;
                     }
                     
                     // Draw Input Line
                     let prompt = format!("user@wasmix:~$ {}_", self.term_input);
                     ui::draw_text(content_x + 5, dy, &prompt, ui::COLOR_WHITE);
                },
                "task_manager" => {
                    // Table Header
                    ui::draw_text(content_x + 5, content_y + 5, "ID  Name        State", ui::COLOR_WHITE);
                    ui::sys_draw_rect(content_x + 5, content_y + 16, content_w - 10, 1, ui::COLOR_DARK_GRAY);
                    
                    if let Some(wins) = windows {
                         let mut dy = content_y + 20;
                         for win in wins {
                              let state = if win.minimized { "Min" } else { "Run" };
                              // Manual formatting assuming monospace font approx 8px
                              // ID: 3 chars, Name: 12 chars, State
                              let line = format!("{:<3} {:<11} {}", win.id, win.title, state);
                              ui::draw_text(content_x + 5, dy, &line, ui::COLOR_WHITE);
                              dy += 16;
                         }
                    }
                    
                    // "End Task" Button area (visual representation)
                    // If we support selection, we would highlight the selected row.
                    // For now, let's keep it simple: Just a list. 
                    // To make it functional (End Task), we need selection state.
                    // Adding `selected_pid: Option<usize>` to Window struct is cleaner.
                    // But for this step, let's just show the list.
                },
                _ => {}
            }
        }
    }
}
