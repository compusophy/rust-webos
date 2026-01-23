#[link(wasm_import_module = "env")]
extern "C" {
    fn sys_print(ptr: *const u8, len: usize);
    fn sys_gpu_width() -> i32;
    fn sys_gpu_height() -> i32;
    fn sys_gpu_clear(r: i32, g: i32, b: i32);
    fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    fn sys_enable_gui_mode();
}

// Colors
const COLOR_DESKTOP: i32 = 0x00_80_80_FFu32 as i32; // Teal
const COLOR_WHITE: i32 = 0xFF_FF_FF_FFu32 as i32;
const COLOR_GRAY: i32 = 0xCC_CC_CC_FFu32 as i32;
const COLOR_DARK_GRAY: i32 = 0x80_80_80_FFu32 as i32;
const COLOR_BLACK: i32 = 0x00_00_00_FFu32 as i32;
const COLOR_BLUE: i32 = 0x00_00_80_FFu32 as i32;

#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        sys_enable_gui_mode();
        let width = sys_gpu_width();
        let height = sys_gpu_height();
        
        // 1. Draw Desktop Background (Teal)
        // Extract RGB from COLOR_DESKTOP
        let r = ((COLOR_DESKTOP >> 24) & 0xFF) as i32;
        let g = ((COLOR_DESKTOP >> 16) & 0xFF) as i32;
        let b = ((COLOR_DESKTOP >> 8) & 0xFF) as i32;
        sys_gpu_clear(r, g, b); 
        
        let msg = "desktop environment loaded";
        sys_print(msg.as_ptr(), msg.len());

        // 2. Draw Taskbar
        let taskbar_height = 40;
        sys_draw_rect(0, height - taskbar_height, width, taskbar_height, COLOR_GRAY);
        
        // Start Menu Button
        sys_draw_rect(2, height - taskbar_height + 2, 60, taskbar_height - 4, COLOR_GRAY);
        // Bevel for button 
        sys_draw_rect(2, height - taskbar_height + 2, 60, 2, COLOR_WHITE); // Top highlight
        sys_draw_rect(2, height - taskbar_height + 2, 2, taskbar_height - 4, COLOR_WHITE); // Left highlight
        sys_draw_rect(60, height - taskbar_height + 2, 2, taskbar_height - 4, COLOR_DARK_GRAY); // Right shadow
        sys_draw_rect(2, height - 2, 60, 2, COLOR_DARK_GRAY); // Bottom shadow
        
        // Window 1: File Manager
        draw_window(50, 50, 300, 200, "File Manager");
        
        // Window 2: Terminal
        draw_window(100, 100, 300, 200, "Terminal");
    }
}

unsafe fn draw_window(x: i32, y: i32, w: i32, h: i32, _title: &str) {
    // Shadow
    sys_draw_rect(x + 5, y + 5, w, h, 0x00_00_00_80); 
    
    // Main Body
    sys_draw_rect(x, y, w, h, COLOR_GRAY);
    
    // Border Check logic? No just draw borders
    
    // Title Bar
    sys_draw_rect(x + 3, y + 3, w - 6, 20, COLOR_BLUE);
    
    // Client Area
    sys_draw_rect(x + 3, y + 26, w - 6, h - 30, COLOR_WHITE);
    
    // TODO: Draw Text title when we have sys_draw_text
}
