use crate::gfx::Context;

#[derive(Clone, Copy)]
pub struct Cell {
    pub c: char,
    pub fg: u32,
    pub bg: u32,
}

pub struct Terminal {
    pub rows: usize,
    pub cols: usize,
    pub buffer: Vec<Cell>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_visible: bool,
    pub default_fg: u32,
    pub default_bg: u32,
}

impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        let default_fg = 0xFF_FF_FF_FF; // White
        let default_bg = 0x00_00_00_FF; // Black
        let buffer = vec![Cell { c: ' ', fg: default_fg, bg: default_bg }; cols * rows];
        Self {
            rows,
            cols,
            buffer,
            cursor_x: 0,
            cursor_y: 0,
            default_fg,
            default_bg,
            cursor_visible: false, // Default hidden for BIOS
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

    pub fn show_cursor(&mut self, visible: bool) {
        self.cursor_visible = visible;
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
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll();
            self.cursor_y = self.rows - 1;
        }
    }

    fn scroll(&mut self) {
        // Remove first row, append empty row
        // Basic implementation: shift everything up
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

    pub fn set_fg_color(&mut self, color: u32) {
        self.default_fg = color;
    }

    pub fn set_bg_color(&mut self, color: u32) {
        self.default_bg = color;
    }

    pub fn render(&self, ctx: &mut Context, offset_x: u32, offset_y: u32) {
        // We assume 8x16 font for now
        // TODO: Import font constants
        let char_w = 8;
        let char_h = 16;
        
        // Fill the left padding (offset) with default background to prevent artifacts
        if offset_x > 0 {
             ctx.fill_rect(0, 0, offset_x, (self.rows * char_h) as u32, self.default_bg);
        }
        
        for y in 0..self.rows {
            for x in 0..self.cols {
                let cell = &self.buffer[y * self.cols + x];
                let draw_x = offset_x + (x * char_w) as u32;
                let draw_y = offset_y + (y * char_h) as u32;

                // Draw background only if opaque
                if (cell.bg & 0xFF) != 0 {
                    ctx.fill_rect(draw_x, draw_y, char_w as u32, char_h as u32, cell.bg);
                }
                
                // Draw char centered vertically in 16px cell (offset +4)
                crate::gfx::font::draw_char(ctx, draw_x, draw_y + 4, cell.c, cell.fg);
            }
        }
        
        // Draw cursor
        if self.cursor_visible {
            let cx = offset_x + (self.cursor_x * char_w) as u32;
            let cy = offset_y + (self.cursor_y * char_h) as u32;
            
            // Vertical Bar Cursor
            // Thicker (4px) and vertically centered on the 16px cell with slight overflow relative to font.
            // Height 14px (y+1 to y+15), leaving 1px top/bottom in 16px cell.
            ctx.fill_rect(cx, cy + 1, 4, 14, 0xFF_FF_FF_FF); 
        }
    }
}
