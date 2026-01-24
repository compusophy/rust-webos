use crate::ui;

#[derive(Clone, Copy)]
pub struct Cell {
    pub c: char,
    pub fg: i32,
    pub bg: i32,
}

pub struct Terminal {
    pub rows: usize,
    pub cols: usize,
    pub buffer: Vec<Cell>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_visible: bool,
    pub default_fg: i32,
    pub default_bg: i32,
}

impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        let default_fg = 0xFF_FF_FF_FFu32 as i32; // White
        let default_bg = 0x00_00_00_FFu32 as i32; // Black
        let buffer = vec![Cell { c: ' ', fg: default_fg, bg: default_bg }; cols * rows];
        Self {
            rows,
            cols,
            buffer,
            cursor_x: 0,
            cursor_y: 0,
            default_fg,
            default_bg,
            cursor_visible: true,
        }
    }

    pub fn reset(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        for cell in self.buffer.iter_mut() {
            cell.c = ' ';
            cell.fg = self.default_fg;
            cell.bg = self.default_bg;
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
            return;
        }
        
        if c == '\r' {
            self.cursor_x = 0;
            return;
        }

        if c == '\x08' { // Backspace
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
            } else if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.cols - 1;
            }
            // Clear the character
            let idx = self.cursor_y * self.cols + self.cursor_x;
            self.buffer[idx].c = ' ';
            return;
        }

        if self.cursor_x >= self.cols {
            self.new_line();
        }

        let idx = self.cursor_y * self.cols + self.cursor_x;
        self.buffer[idx].c = c;
        self.buffer[idx].fg = self.default_fg;
        self.buffer[idx].bg = self.default_bg;

        self.cursor_x += 1;
    }

    fn new_line(&mut self) {
        // "Lines above turn white" logic REMOVED.
        // History keeps original colors.

        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll();
            self.cursor_y = self.rows - 1;
        }
    }

    fn scroll(&mut self) {
        // Remove first row, append empty row
        for y in 0..self.rows - 1 {
            for x in 0..self.cols {
                let src_idx = (y + 1) * self.cols + x;
                let dst_idx = y * self.cols + x;
                self.buffer[dst_idx] = self.buffer[src_idx];
            }
        }
        // Clear last row
        let start = (self.rows - 1) * self.cols;
        for i in 0..self.cols {
            self.buffer[start + i] = Cell {
                c: ' ',
                fg: self.default_fg,
                bg: self.default_bg,
            };
        }
    }

    pub fn set_fg_color(&mut self, color: i32) {
        self.default_fg = color;
    }

    pub fn set_bg_color(&mut self, color: i32) {
        self.default_bg = color;
    }

    pub fn render(&self, offset_x: i32, offset_y: i32) {
        // We assume 8x16 font
        let char_w = 8;
        let char_h = 16;
        
        // Use batched rendering optimization?
        // Simple approach first: char by char.
        // sys_draw_text takes a string. We can pass 1-char string.
        
        for y in 0..self.rows {
            for x in 0..self.cols {
                let cell = &self.buffer[y * self.cols + x];
                
                // Optimization: Skip empty cells if bg is black/transparent
                if cell.c == ' ' && cell.bg == 0 {
                    continue; 
                }

                let draw_x = offset_x + (x as i32 * char_w);
                let draw_y = offset_y + (y as i32 * char_h);

                // Draw background
                if (cell.bg & 0xFF) != 0 {
                    ui::draw_rect(draw_x, draw_y, char_w, char_h, cell.bg);
                }
                
                // Draw char centered vertically +4px
                if cell.c != ' ' {
                    // String allocation per char is suboptimal but robust for now.
                    // Could optimize by gathering contiguous runs.
                    let s = cell.c.to_string(); 
                    ui::draw_text(draw_x, draw_y + 4, &s, cell.fg);
                }
            }
        }
        
        // Draw cursor
        if self.cursor_visible {
            let cx = offset_x + (self.cursor_x as i32 * char_w);
            let cy = offset_y + (self.cursor_y as i32 * char_h);
            
            // Vertical Bar Cursor: 4px width, 14px height
            ui::draw_rect(cx, cy + 1, 4, 14, 0xFF_FF_FF_FFu32 as i32); 
        }
    }
}
