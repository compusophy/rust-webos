use crate::ui;
use crate::window::Window;

extern "C" {
    fn sys_reset();
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
             let menu_h = 200;
             let menu_x = 2;
             let menu_y = taskbar_y - menu_h;
             
             if x >= menu_x && x < menu_x + menu_w && y >= menu_y && y < menu_y + menu_h {
                 // Handle Click
                 let rel_y = y - menu_y;
                 if rel_y >= 10 && rel_y <= 30 {
                     self.spawn_window("Terminal", "terminal");
                     self.start_menu_open = false;
                 } else if rel_y >= 30 && rel_y <= 50 {
                     self.spawn_window("File Manager", "file_manager");
                     self.start_menu_open = false;
                 } else if rel_y >= 70 && rel_y <= 90 { // Shutdown/Reset
                      unsafe { sys_reset(); }
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
            }
            // Todo: Taskbar Items click to focus/minimize
            return;
        }

        // 3. Check Windows (Top -> Down)
        let mut hit_idx = None;
        for (i, win) in self.windows.iter_mut().enumerate().rev() {
            if win.contains(x, y) {
                hit_idx = Some(i);
                
                // Close Button
                if win.close_button_contains(x, y) {
                    // We remove it later to avoid index issues
                    // Or we mark a flag?
                    // Let's return a "Action" enum? 
                    // For now, let's just cheat: we can't remove while iterating mutable.
                    // We'll modify a "close_requested" list?
                    // Actually, we can just break and remove it outside.
                } else if win.title_bar_contains(x, y) {
                    win.is_dragging = true;
                    win.drag_offset_x = x - win.x;
                    win.drag_offset_y = y - win.y;
                }
                break;
            }
        }
        
        if let Some(idx) = hit_idx {
            // Check if we hit the close button
            if self.windows[idx].close_button_contains(x, y) {
                self.windows.remove(idx);
                self.active_window_idx = None;
            } else {
                // Focus
                self.active_window_idx = Some(idx);
                // Move to end (top)
                let win = self.windows.remove(idx);
                self.windows.push(win);
                self.active_window_idx = Some(self.windows.len() - 1);
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

    pub fn draw(&self) {
        unsafe {
            // Desktop BG
            ui::sys_draw_rect(0, 0, self.width, self.height, ui::COLOR_DESKTOP);
            
            // Taskbar
            let tb_h = 40;
            let tb_y = self.height - tb_h;
            ui::draw_panel_raised(0, tb_y, self.width, tb_h);
            
            // Start Button
            ui::draw_button(2, tb_y + 4, 60, 32, "Start", self.start_menu_open);
            
            // Taskbar Items
            let mut start_x = 70;
            for (i, win) in self.windows.iter().enumerate() {
                 let is_active = self.active_window_idx == Some(i);
                 // Draw button
                 let width = 100;
                 if is_active {
                     ui::draw_panel_sunken(start_x, tb_y + 4, width, 32, ui::COLOR_WHITE);
                     ui::draw_text(start_x + 6, tb_y + 12, &win.title, ui::COLOR_BLACK);
                 } else {
                     ui::draw_panel_raised(start_x, tb_y + 4, width, 32);
                     ui::draw_text(start_x + 6, tb_y + 12, &win.title, ui::COLOR_BLACK);
                 }
                 start_x += width + 5;
            }

            // Windows
            for (i, win) in self.windows.iter().enumerate() {
                win.draw(self.active_window_idx == Some(i));
            }
            
            // Start Menu
            if self.start_menu_open {
                 let menu_w = 150;
                 let menu_h = 200;
                 let menu_y = tb_y - menu_h;
                 let menu_x = 2;
                 
                 ui::draw_panel_raised(menu_x, menu_y, menu_w, menu_h);
                 ui::sys_draw_rect(menu_x + 3, menu_y + 3, 20, menu_h - 6, ui::COLOR_NAVY);
                 
                 // Items
                 // TODO: Hover effects
                 // 1. Programs
                 let row_h = 20;
                 let mut draw_y = menu_y + 10;
                 
                 let items = ["Programs", "Documents", "Settings", "Reset"];
                 
                 for item in items.iter() {
                     // Hit Test for hover (Visual only)
                     if self.mouse_x >= menu_x && self.mouse_x < menu_x + menu_w && 
                        self.mouse_y >= draw_y && self.mouse_y < draw_y + row_h {
                             ui::sys_draw_rect(menu_x + 25, draw_y, menu_w - 30, row_h, ui::COLOR_HIGHLIGHT);
                             ui::draw_text(menu_x + 30, draw_y + 4, item, ui::COLOR_WHITE);
                     } else {
                             ui::draw_text(menu_x + 30, draw_y + 4, item, ui::COLOR_BLACK);
                     }
                     draw_y += row_h;
                 }
            }
        }
    }
}
