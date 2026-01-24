use crate::ui;
use crate::shell::Shell;

pub struct Window {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub title: String,
    pub content_type: String, // "file_manager", "terminal", "task_manager"
    pub is_dragging: bool,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    pub minimized: bool,
    
    // State for File Manager
    pub current_path: String,
    pub files: Vec<(bool, String)>,
    
    // State for Terminal
    pub shell: Option<Shell>,
    
    // State for Task Manager
    pub selected_pid: Option<usize>,
}

impl Window {
    pub fn new(id: usize, x: i32, y: i32, w: i32, h: i32, title: &str, content_type: &str) -> Self {
        let mut shell = None;
        if content_type == "terminal" {
            // Calculate cols/rows based on window size
            // Padding: Left 4, Right 4 => -8 width
            // Header: 24, Bottom: 4 => -28 height
            // Char: 8x16
            let cols = (w - 8) / 8;
            let rows = (h - 28) / 16;
             // Ensure at least 1x1
            let cols = if cols > 0 { cols as usize } else { 1 };
            let rows = if rows > 0 { rows as usize } else { 1 };
            
            shell = Some(Shell::new(cols, rows));
        }

        let mut win = Self {
            id,
            x, y, w, h,
            title: title.to_string(),
            content_type: content_type.to_string(),
            is_dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
            minimized: false,
            current_path: "/".to_string(),
            files: Vec::new(),
            shell,
            selected_pid: None,
        };
        
        if content_type == "file_manager" {
            win.refresh_files();
        }
        
        win
    }
    
    pub fn on_key(&mut self, code: u32) {
        if self.minimized { return; }
        if self.content_type == "terminal" {
            if let Some(shell) = &mut self.shell {
                shell.on_key(code);
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
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + 25
    }

    pub fn close_button_contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        let btn_x = self.x + self.w - 20;
        let btn_y = self.y + 5;
        let btn_w = 14;
        let btn_h = 14;
        x >= btn_x && x < btn_x + btn_w && y >= btn_y && y < btn_y + btn_h
    }
    
    pub fn minimize_button_contains(&self, x: i32, y: i32) -> bool {
        if self.minimized { return false; }
        let btn_x = self.x + self.w - 38;
        let btn_y = self.y + 5;
        let btn_w = 14;
        let btn_h = 14;
        x >= btn_x && x < btn_x + btn_w && y >= btn_y && y < btn_y + btn_h
    }

    pub fn draw(&self, is_active: bool, windows: Option<&Vec<Window>>) {
        if self.minimized { return; }
        
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
            
            let bg = if self.content_type == "terminal" { ui::COLOR_BLACK } else { ui::COLOR_GRAY }; 
            ui::draw_panel_sunken(content_x, content_y, content_w, content_h, bg);
            
            // Draw Context
            match self.content_type.as_str() {
                "file_manager" => {
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
                     if let Some(shell) = &self.shell {
                         shell.draw(content_x, content_y);
                     }
                },
                "task_manager" => {
                    // Table Header
                    // Table Header
                    ui::draw_text(content_x + 5, content_y + 5, "id  name        state", ui::COLOR_WHITE);
                    ui::sys_draw_rect(content_x + 5, content_y + 16, content_w - 10, 1, ui::COLOR_DARK_GRAY);
                    
                    if let Some(wins) = windows {
                         let mut dy = content_y + 20;
                         for win in wins {
                              let state = if win.minimized { "min" } else { "run" };
                              let line = format!("{:<3} {:<11} {}", win.id, win.title, state);
                              
                              // Highlight Selection
                              if Some(win.id) == self.selected_pid {
                                  ui::sys_draw_rect(content_x + 2, dy - 2, content_w - 4, 16, ui::COLOR_HIGHLIGHT);
                              }
                              
                              ui::draw_text(content_x + 5, dy, &line, ui::COLOR_WHITE);
                              dy += 16;
                         }
                    }

                    // End Task Button (Bottom Right)
                    let btn_w = 80;
                    let btn_h = 24;
                    let btn_x = content_x + content_w - btn_w - 5;
                    let btn_y = content_y + content_h - btn_h - 5;
                    
                    // Only active if selection is valid
                    if self.selected_pid.is_some() {
                        ui::draw_button(btn_x, btn_y, btn_w, btn_h, "end task", false);
                    }
                },
                _ => {}
            }
        }
    }
}
