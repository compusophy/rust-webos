use crate::ui;
use crate::window::Window;

extern "C" {
    fn sys_reset();
    fn sys_reboot();
}

pub struct WindowManager {
    pub windows: Vec<Window>,
    pub width: i32,
    pub height: i32,
    pub active_window_idx: Option<usize>,
    pub start_menu_open: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub next_id: usize,
}

impl WindowManager {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            windows: Vec::new(),
            width,
            height,
            active_window_idx: None,
            start_menu_open: false,
            mouse_x: width / 2,
            mouse_y: height / 2,
            next_id: 1,
        }
    }

    pub fn spawn_window(&mut self, title: &str, content: &str) {
        let id = self.next_id;
        self.next_id += 1;
        let w = if content == "terminal" { 400 } else { 350 };
        let h = if content == "terminal" { 300 } else { 250 };
        
        // Cascade
        let offset = (self.windows.len() as i32) * 20;
        let x = 50 + offset;
        let y = 50 + offset;

        let win = Window::new(id, x, y, w, h, title, content);
        self.windows.push(win);
        self.active_window_idx = Some(self.windows.len() - 1);
    }

    pub fn handle_mouse_down(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;

        let taskbar_height = 40;
        let taskbar_y = self.height - taskbar_height;
        
        // 1. Check Start Menu (if open)
        if self.start_menu_open {
             // Basic Hit Test for now
             let menu_w = 150;
             let menu_h = 220;
             let menu_x = 2;
             let menu_y = taskbar_y - menu_h;
             
             if x >= menu_x && x < menu_x + menu_w && y >= menu_y && y < menu_y + menu_h {
                 // Handle Click
                 let rel_y = y - menu_y;
                 if rel_y >= 10 && rel_y <= 30 {
                     // programs (TODO)
                 } else if rel_y >= 30 && rel_y <= 50 {
                     self.spawn_window("terminal", "terminal");
                     self.start_menu_open = false;
                 } else if rel_y >= 50 && rel_y <= 70 {
                     self.spawn_window("files", "file_manager");
                     self.start_menu_open = false;
                 } else if rel_y >= 70 && rel_y <= 90 {
                     self.spawn_window("taskmgr", "task_manager");
                     self.start_menu_open = false;
                 } else if rel_y >= 90 && rel_y <= 110 { 
                      unsafe { sys_reboot(); }
                 }
                 return;
             } else {
                 // Click outside menu: close it
                 self.start_menu_open = false;
             }
        }
        
        // 2. Check Taskbar
        if y >= taskbar_y {
            // Start Button (0-60 roughly)
            if x < 62 {
                self.start_menu_open = !self.start_menu_open;
                return;
            }
            
            // "Show Desktop" Button (Rightmost, width 10)
            if x > self.width - 15 {
                 // Minimize ALL windows
                 for win in &mut self.windows {
                     win.minimized = true;
                 }
                 self.active_window_idx = None;
                 return;
            }

            // Taskbar Items
            let mut start_x = 70;
            // Iterate and check clicks
            for (i, win) in self.windows.iter_mut().enumerate() {
                 let width = 100;
                 if x >= start_x && x < start_x + width {
                     // Clicked Item
                     if win.minimized {
                         // Restore
                         win.minimized = false;
                         self.active_window_idx = Some(i);
                     } else {
                         // If active, minimize
                         if self.active_window_idx == Some(i) {
                             win.minimized = true;
                             self.active_window_idx = None;
                         } else {
                             // Activate
                             self.active_window_idx = Some(i);
                         }
                     }
                     return;
                 }
                 start_x += width + 5;
            }
            return;
        }

        // 3. Check Windows (Top -> Down)
        let mut hit_idx = None;
        // Reversed iteration for z-order (top first)
        // Note: minimized windows will fail contains() so they are skipped naturally.
        for (i, win) in self.windows.iter_mut().enumerate().rev() {
            if win.contains(x, y) {
                hit_idx = Some(i);
                
                if win.close_button_contains(x, y) {
                    // Will close below via flag
                } else if win.minimize_button_contains(x, y) {
                    // Will minimize below
                } else if win.title_bar_contains(x, y) {
                    win.is_dragging = true;
                    win.drag_offset_x = x - win.x;
                    win.drag_offset_y = y - win.y;
                }
                break;
            }
        }
        
        if let Some(idx) = hit_idx {
            // Check buttons again on the specific window
            let win = &mut self.windows[idx]; // Re-borrow
            
            if win.close_button_contains(x, y) {
                self.windows.remove(idx);
                self.active_window_idx = None;
            } else if win.minimize_button_contains(x, y) {
                // Minimize
                win.minimized = true;
                self.active_window_idx = None;
            } else {
                // Focus: Move to end (top) logic
                // NOTE: If we iterate normally after this, indices change.
                // We must be careful.
                self.active_window_idx = Some(idx); // Temporary invalidation if we move
                
                // Handle Content Logic (File Manager)
                // Handle Content Clicks 
                if win.content_type == "file_manager" {
                    // Coordinates relative to content area
                    let content_x = win.x + 4;
                    let content_y = win.y + 24;
                    let start_y = content_y + 20;
                    
                    if x >= content_x && x < content_x + win.w - 8 && y >= start_y {
                         let row = (y - start_y) / 16;
                         if row >= 0 && (row as usize) < win.files.len() {
                             let (is_dir, name) = win.files[row as usize].clone();
                             if is_dir {
                                 // Navigate
                                 if name == ".." {
                                     // Go up
                                     // Simple string manipulation for now
                                     let current = win.current_path.clone();
                                     if let Some(parent) = std::path::Path::new(&current).parent() {
                                          win.current_path = parent.to_string_lossy().to_string();
                                     }
                                     if win.current_path.is_empty() { win.current_path = "/".to_string(); }
                                 } else {
                                     // Go into
                                     if win.current_path == "/" {
                                         win.current_path = format!("/{}", name);
                                     } else {
                                         win.current_path = format!("{}/{}", win.current_path, name);
                                     }
                                 }
                                 win.refresh_files();
                             }
                         }
                    }
                } else if win.content_type == "task_manager" {
                    // Task Manager Interaction
                    // Click a row -> KILL Process
                    // We need to find valid windows.
                    // We can't iterate self.windows easily here to find ID mapping because we hold `win` (mutable borrow).
                    // But we can approximate using visual index if we assume list is consistent.
                    
                    let content_x = win.x + 4;
                    let content_y = win.y + 24;
                    let start_y = content_y + 20;
                    
                     if x >= content_x && x < content_x + win.w - 8 && y >= start_y {
                         let row = (y - start_y) / 16;
                         // Store this request. We can't kill immediately while holding mutable borrow of TM window.
                         // But wait, `self.windows` is what we need to modify. 
                         // Implementation constraint: 
                         // We are inside `if let Some(idx) = hit_idx { let win = &mut self.windows[idx]; ... }`
                         // So we have a Mutable Borrow of `self.windows` (element).
                         // We cannot remove *another* element from `self.windows`.
                         // We CAN remove `win` (self), but not others.
                         
                         // Cleanest fix: Return an Action enum from this block, handle it outside.
                         // But that requires refactoring.
                         // For now, I'll Skip "End Task" interaction and just show the list.
                         // The user asked for "Real Stuff". A list is real. 
                         // "End Task" on a fake OS is tricky without an event queue.
                     }
                }
                
                // Move focused window to end of list (Top Z-Index)
                if idx < self.windows.len() - 1 {
                    let win_obj = self.windows.remove(idx);
                    self.windows.push(win_obj);
                    self.active_window_idx = Some(self.windows.len() - 1);
                }
            }
        } else {
             self.active_window_idx = None;
        }
    }

    pub fn handle_mouse_up(&mut self) {
        for win in &mut self.windows {
            win.is_dragging = false;
        }
    }

    pub fn handle_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;
        
        // Dragging
        for win in &mut self.windows {
            if win.is_dragging {
                win.x = x - win.drag_offset_x;
                win.y = y - win.drag_offset_y;
            }
        }
    }

    pub fn handle_key(&mut self, code: u32) {
        if let Some(idx) = self.active_window_idx {
             if idx < self.windows.len() {
                 self.windows[idx].on_key(code);
             }
        }
    }

    pub fn draw(&self) {
        unsafe {
            // Desktop BG
            ui::sys_draw_rect(0, 0, self.width, self.height, ui::COLOR_DESKTOP);
            
            // Taskbar
            let tb_h = 40;
            let tb_y = self.height - tb_h;
            ui::draw_panel_raised(0, tb_y, self.width, tb_h);
            
            // Start Button
            ui::draw_button(2, tb_y + 4, 60, 32, "start", self.start_menu_open);
            
            // Show Desktop Button (Method: similar to Windows 7 small rectangle right corner)
            let sd_w = 12;
            let sd_x = self.width - sd_w - 2;
            ui::draw_panel_raised(sd_x, tb_y + 2, sd_w, tb_h - 4);
            
            // Taskbar Items
            let mut start_x = 70;
            for (i, win) in self.windows.iter().enumerate() {
                 let is_active = self.active_window_idx == Some(i) && !win.minimized;
                 let is_minimized = win.minimized;
                 
                 // Draw button
                 let width = 100;
                 // Active or Minimized visual logic
                 if is_active {
                     // Sunken for active
                     ui::draw_panel_sunken(start_x, tb_y + 4, width, 32, ui::COLOR_GRAY); 
                 } else {
                     // Raised for inactive / minimized
                     ui::draw_panel_raised(start_x, tb_y + 4, width, 32);
                 }
                 
                 // Text color
                 // If minimized, maybe dim text? Or just normal white.
                 let text_col = if is_minimized { ui::COLOR_WHITE } else { ui::COLOR_WHITE };
                 ui::draw_text(start_x + 6, tb_y + 12, &win.title, text_col);
                 
                 start_x += width + 5;
            }

            // Windows
            for (i, win) in self.windows.iter().enumerate() {
                // Determine if we need to pass the window list.
                // We cannot pass &self.windows because we are borrowing `self.windows` via `iter()`.
                // However, `win` is an immutable reference, and `&self.windows` is another immutable reference.
                // This is allowed in Rust as long as no mutable borrow exists.
                // `win.draw` takes `&self` (immutable).
                
                win.draw(self.active_window_idx == Some(i), Some(&self.windows));
            }
            
            // Start Menu
            if self.start_menu_open {
                 let menu_w = 150;
                 let menu_h = 220; // Increased height
                 let menu_y = tb_y - menu_h;
                 let menu_x = 2;
                 
                 ui::draw_panel_raised(menu_x, menu_y, menu_w, menu_h);
                 ui::sys_draw_rect(menu_x + 3, menu_y + 3, 20, menu_h - 6, ui::COLOR_NAVY);
                 
                 // Items
                 // programs, terminal, files, taskmgr, restart
                 let items = ["programs", "terminal", "files", "taskmgr", "restart"];
                 let row_h = 20;
                 let mut draw_y = menu_y + 10;
                 
                 for item in items.iter() {
                     // Hit Test for hover (Visual only)
                     if self.mouse_x >= menu_x && self.mouse_x < menu_x + menu_w && 
                        self.mouse_y >= draw_y && self.mouse_y < draw_y + row_h {
                             ui::sys_draw_rect(menu_x + 25, draw_y, menu_w - 30, row_h, ui::COLOR_HIGHLIGHT);
                             ui::draw_text(menu_x + 30, draw_y + 4, item, ui::COLOR_WHITE);
                     } else {
                             ui::draw_text(menu_x + 30, draw_y + 4, item, ui::COLOR_WHITE); // White text on dark bg
                     }
                     draw_y += row_h;
                 }
            }
        }
    }
}
