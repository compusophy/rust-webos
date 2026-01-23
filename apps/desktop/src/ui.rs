
// UI Theme Constants
// UI Theme Constants (Modern Dark)
pub const COLOR_DESKTOP: i32 = 0x1A_1B_26_FFu32 as i32; // Tokyo Night Dark
pub const COLOR_WHITE: i32 = 0xA9_B1_D6_FFu32 as i32; // Text Color
pub const COLOR_GRAY: i32 = 0x24_28_3B_FFu32 as i32; // Panel BG
pub const COLOR_DARK_GRAY: i32 = 0x41_48_68_FFu32 as i32; // Borders
pub const COLOR_BLUE: i32 = 0x7A_A2_F7_FFu32 as i32; // Active Title/Accent
pub const COLOR_NAVY: i32 = 0x24_28_3B_FFu32 as i32; // Inactive Title
pub const COLOR_BLACK: i32 = 0x15_16_1E_FFu32 as i32; // Shadow/Black
pub const COLOR_HIGHLIGHT: i32 = 0x3D_59_A1_FFu32 as i32; // Hover

extern "C" {
    pub fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    pub fn sys_draw_text(ptr: *const u8, len: usize, x: i32, y: i32, color: i32);
    pub fn sys_fs_list(path_ptr: *const u8, path_len: i32, out_ptr: *mut u8, out_len: i32) -> i32;
    pub fn sys_exec(cmd_ptr: *const u8, cmd_len: i32, out_ptr: *mut u8, out_len: i32) -> i32;
}

pub fn read_dir(path: &str) -> Vec<(bool, String)> {
    let mut buf = [0u8; 4096];
    let res = unsafe {
        sys_fs_list(path.as_ptr(), path.len() as i32, buf.as_mut_ptr(), 4096)
    };
    
    if res > 0 {
        if let Ok(s) = std::str::from_utf8(&buf[0..res as usize]) {
             let mut entries = Vec::new();
             for line in s.lines() {
                 if let Some((type_char, name)) = line.split_once(':') {
                     entries.push((type_char == "D", name.to_string()));
                 }
             }
             return entries;
        }
    }
    Vec::new()
}

pub fn draw_text(x: i32, y: i32, text: &str, color: i32) {
    unsafe {
        sys_draw_text(text.as_ptr(), text.len(), x, y, color);
    }
}

pub unsafe fn draw_panel_raised(x: i32, y: i32, w: i32, h: i32) {
    // Flat style calling for modern look
    sys_draw_rect(x, y, w, h, COLOR_GRAY);
    sys_draw_rect(x, y, w, 1, COLOR_DARK_GRAY); // Top Border
    sys_draw_rect(x, y, 1, h, COLOR_DARK_GRAY); // Left Border
    sys_draw_rect(x + w - 1, y, 1, h, COLOR_BLACK); // Right Border
    sys_draw_rect(x, y + h - 1, w, 1, COLOR_BLACK); // Bottom Border
}

pub unsafe fn draw_panel_sunken(x: i32, y: i32, w: i32, h: i32, bg: i32) {
    sys_draw_rect(x, y, w, h, bg);
    sys_draw_rect(x, y, w, 1, COLOR_BLACK);
    sys_draw_rect(x, y, 1, h, COLOR_BLACK);
    sys_draw_rect(x + w - 1, y, 1, h, COLOR_DARK_GRAY);
    sys_draw_rect(x, y + h - 1, w, 1, COLOR_DARK_GRAY);
}

pub unsafe fn draw_button(x: i32, y: i32, w: i32, h: i32, text: &str, pressed: bool) {
    let text_len = text.len() as i32 * 8;
    let text_x = x + (w - text_len) / 2;
    let text_y = y + (h - 8) / 2;

    if pressed {
        // Pressed State with flat modern style
        sys_draw_rect(x, y, w, h, COLOR_HIGHLIGHT); 
        sys_draw_rect(x, y, w, 1, COLOR_BLACK);
        sys_draw_rect(x, y, 1, h, COLOR_BLACK);
        // Text centered
        draw_text(text_x + 1, text_y + 1, text, COLOR_WHITE);
    } else {
        // Normal State
        draw_panel_raised(x, y, w, h);
        draw_text(text_x, text_y, text, COLOR_WHITE);
    }
}

pub fn exec(cmd: &str) -> String {
    let mut out_buf = [0u8; 4096];
    let res = unsafe {
        sys_exec(cmd.as_ptr(), cmd.len() as i32, out_buf.as_mut_ptr(), 4096)
    };
    
    if res >= 0 {
        let s = std::str::from_utf8(&out_buf[0..res as usize]).unwrap_or("");
        s.to_string()
    } else {
        format!("Error executing command: {}", cmd)
    }
}
