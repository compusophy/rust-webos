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
}

impl Window {
    pub fn new(id: usize, x: i32, y: i32, w: i32, h: i32, title: &str, content_type: &str) -> Self {
        Self {
            id,
            x, y, w, h,
            title: title.to_string(),
            content_type: content_type.to_string(),
            is_dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn title_bar_contains(&self, x: i32, y: i32) -> bool {
        // Title bar is 22px height approx (including borders)
        // From drawing code: y..y+25 area generally
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + 25
    }

    pub fn close_button_contains(&self, x: i32, y: i32) -> bool {
        // Close button: right aligned.
        let btn_x = self.x + self.w - 20;
        let btn_y = self.y + 5;
        let btn_w = 14;
        let btn_h = 14;
        x >= btn_x && x < btn_x + btn_w && y >= btn_y && y < btn_y + btn_h
    }

    pub fn draw(&self, is_active: bool) {
        unsafe {
             // Shadow
            ui::sys_draw_rect(self.x + 5, self.y + 5, self.w, self.h, 0x00_00_00_80); 
            
            // Frame
            ui::draw_panel_raised(self.x, self.y, self.w, self.h);

            // Title Bar
            let title_color = if is_active { ui::COLOR_NAVY } else { ui::COLOR_DARK_GRAY };
            ui::sys_draw_rect(self.x + 3, self.y + 3, self.w - 6, 18, title_color);
            
            // Title Text
            ui::draw_text(self.x + 6, self.y + 6, &self.title, ui::COLOR_WHITE);
            
            // Close Button
            let btn_x = self.x + self.w - 20;
            let btn_y = self.y + 5;
            ui::draw_panel_raised(btn_x, btn_y, 14, 14);
            ui::draw_text(btn_x + 4, btn_y + 1, "x", ui::COLOR_BLACK);

            // Content Area
            let content_x = self.x + 4;
            let content_y = self.y + 24;
            let content_w = self.w - 8;
            let content_h = self.h - 28;
            
            // White bg for content usually
            let bg = if self.content_type == "terminal" { ui::COLOR_BLACK } else { ui::COLOR_WHITE };
            ui::draw_panel_sunken(content_x, content_y, content_w, content_h, bg);
            
            // Draw Context
            match self.content_type.as_str() {
                "file_manager" => {
                    ui::draw_text(content_x + 10, content_y + 10, "[DIR] bin", ui::COLOR_BLACK);
                    ui::draw_text(content_x + 10, content_y + 30, "[DIR] usr", ui::COLOR_BLACK);
                    ui::draw_text(content_x + 10, content_y + 50, "[DIR] home", ui::COLOR_BLACK);
                    ui::draw_text(content_x + 10, content_y + 70, "README.md", ui::COLOR_BLACK);
                },
                "terminal" => {
                    ui::draw_text(content_x + 5, content_y + 5, "user@wasmix:~ $ ls", ui::COLOR_WHITE);
                    ui::draw_text(content_x + 5, content_y + 25, "bin usr home README.md", ui::COLOR_WHITE);
                    ui::draw_text(content_x + 5, content_y + 45, "user@wasmix:~ $ _", ui::COLOR_WHITE);
                },
                _ => {}
            }
        }
    }
}
