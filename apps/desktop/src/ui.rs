
// UI Theme Constants
pub const COLOR_DESKTOP: i32 = 0x00_80_80_FFu32 as i32; // Teal
pub const COLOR_WHITE: i32 = 0xFF_FF_FF_FFu32 as i32;
pub const COLOR_GRAY: i32 = 0xCC_CC_CC_FFu32 as i32;
pub const COLOR_DARK_GRAY: i32 = 0x80_80_80_FFu32 as i32;
pub const COLOR_BLUE: i32 = 0x00_00_80_FFu32 as i32; // Active Title
pub const COLOR_NAVY: i32 = 0x00_00_40_FFu32 as i32; 
pub const COLOR_BLACK: i32 = 0x00_00_00_FFu32 as i32;
pub const COLOR_HIGHLIGHT: i32 = 0x00_00_AA_FFu32 as i32; // Hover/Select

extern "C" {
    pub fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    pub fn sys_draw_text(ptr: *const u8, len: usize, x: i32, y: i32, color: i32);
}

pub fn draw_text(x: i32, y: i32, text: &str, color: i32) {
    unsafe {
        sys_draw_text(text.as_ptr(), text.len(), x, y, color);
    }
}

pub unsafe fn draw_panel_raised(x: i32, y: i32, w: i32, h: i32) {
    sys_draw_rect(x, y, w, h, COLOR_GRAY);
    sys_draw_rect(x, y, w, 1, COLOR_WHITE);
    sys_draw_rect(x, y, 1, h, COLOR_WHITE);
    sys_draw_rect(x + w - 1, y, 1, h, COLOR_DARK_GRAY);
    sys_draw_rect(x, y + h - 1, w, 1, COLOR_DARK_GRAY);
}

pub unsafe fn draw_panel_sunken(x: i32, y: i32, w: i32, h: i32, bg: i32) {
    sys_draw_rect(x, y, w, h, bg);
    sys_draw_rect(x, y, w, 1, COLOR_DARK_GRAY);
    sys_draw_rect(x, y, 1, h, COLOR_DARK_GRAY);
    sys_draw_rect(x + w - 1, y, 1, h, COLOR_WHITE);
    sys_draw_rect(x, y + h - 1, w, 1, COLOR_WHITE);
}

pub unsafe fn draw_button(x: i32, y: i32, w: i32, h: i32, text: &str, pressed: bool) {
    if pressed {
        // Pressed State
        sys_draw_rect(x, y, w, h, COLOR_WHITE);
        sys_draw_rect(x + 1, y + 1, w - 2, h - 2, COLOR_GRAY);
        sys_draw_rect(x + 2, y + 2, w - 4, h - 4, COLOR_DARK_GRAY); // Inner Shadow
        // Text shifted
        draw_text(x + 8 + 2, y + 8 + 2, text, COLOR_BLACK);
    } else {
        // Normal State
        draw_panel_raised(x, y, w, h);
        draw_text(x + 8, y + 8, text, COLOR_BLACK);
    }
}
