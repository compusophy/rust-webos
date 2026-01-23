use std::ffi::CString;

// System Calls
extern "C" {
    fn sys_print(ptr: *const u8, len: usize);
    fn sys_gpu_width() -> i32;
    fn sys_gpu_height() -> i32;
    fn sys_gpu_clear(r: i32, g: i32, b: i32);
    fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    fn sys_draw_text(ptr: *const u8, len: usize, x: i32, y: i32, color: i32);
    fn sys_enable_gui_mode();
    fn sys_poll_event(ptr: *mut u8) -> i32;
}

fn draw_text(x: i32, y: i32, text: &str, color: i32) {
    unsafe {
        sys_draw_text(text.as_ptr(), text.len(), x, y, color);
    }
}

#[repr(C, packed)]
struct SystemEvent {
    event_type: u32,
    code: u32,
    x: i32,
    y: i32,
}

const COLOR_DESKTOP: i32 = 0x00_80_80_FFu32 as i32;
const COLOR_WHITE: i32 = 0xFF_FF_FF_FFu32 as i32;
const COLOR_GRAY: i32 = 0xCC_CC_CC_FFu32 as i32;
const COLOR_DARK_GRAY: i32 = 0x80_80_80_FFu32 as i32;
const COLOR_BLUE: i32 = 0x00_00_80_FFu32 as i32;
const COLOR_BLACK: i32 = 0x00_00_00_FFu32 as i32;
const COLOR_NAVY: i32 = 0x00_00_40_FFu32 as i32;

struct Window {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: String,
    content_type: String, // "file_manager", "terminal", "generic"
    is_dragging: bool,
    drag_offset_x: i32,
    drag_offset_y: i32,
}

struct DesktopState {
    width: i32,
    height: i32,
    windows: Vec<Window>,
    cursor_x: i32,
    cursor_y: i32,
    mouse_down: bool,
    start_menu_open: bool,
    active_window_idx: Option<usize>,
}

impl DesktopState {
    fn new() -> Self {
        let width = unsafe { sys_gpu_width() };
        let height = unsafe { sys_gpu_height() };
        
        let windows = Vec::new();
        // Default windows removed as per user request
        // windows.push(...);

        Self {
            width,
            height,
            windows,
            cursor_x: width / 2,
            cursor_y: height / 2,
            mouse_down: false,
            start_menu_open: false,
            active_window_idx: None,
        }
    }
    
     fn update(&mut self, event: SystemEvent) {
        match event.event_type {
            3 => { // MouseDown
                self.mouse_down = true;
                
                let taskbar_y = self.height - 40;
                
                // Handle Start Menu Interactions
                if self.start_menu_open {
                    let menu_w = 150;
                    let menu_h = 200;
                    let menu_x = 2;
                    let menu_y = taskbar_y - menu_h;
                    
                    // Check if clicked inside menu
                    if self.cursor_x >= menu_x && self.cursor_x <= menu_x + menu_w &&
                       self.cursor_y >= menu_y && self.cursor_y <= menu_y + menu_h {
                        
                        // Simple hit test for items (20px height approx)
                        let rel_y = self.cursor_y - menu_y;
                        
                        // "Programs" (Terminal) -> y=10..30
                        if rel_y >= 10 && rel_y <= 30 {
                             self.windows.push(Window {
                                x: 100, y: 100, w: 400, h: 300,
                                title: "Terminal".to_string(),
                                content_type: "terminal".to_string(),
                                is_dragging: false, drag_offset_x: 0, drag_offset_y: 0,
                            });
                            self.active_window_idx = Some(self.windows.len() - 1);
                            self.start_menu_open = false;
                            return;
                        }
                        // "Documents" (File Manager) -> y=30..50
                        if rel_y >= 30 && rel_y <= 50 {
                             self.windows.push(Window {
                                x: 50, y: 50, w: 400, h: 300,
                                title: "File Manager".to_string(),
                                content_type: "file_manager".to_string(),
                                is_dragging: false, drag_offset_x: 0, drag_offset_y: 0,
                            });
                            self.active_window_idx = Some(self.windows.len() - 1);
                            self.start_menu_open = false;
                            return;
                        }
                        
                        return; // Clicked in menu but no item
                    }
                }

                // Handle Start Button Click
                if self.cursor_y >= taskbar_y {
                    if self.cursor_x >= 2 && self.cursor_x <= 62 {
                        self.start_menu_open = !self.start_menu_open;
                        return;
                    }
                } else {
                    if self.start_menu_open {
                        // Click outside closes menu
                        self.start_menu_open = false;
                    }
                }

                // Check collisions (reverse order for z-index)
                let mut clicked_idx = None;
                for (i, win) in self.windows.iter_mut().enumerate().rev() {
                    // Window Hit Test (Title Bar + Body)
                    if self.cursor_x >= win.x && self.cursor_x <= win.x + win.w &&
                       self.cursor_y >= win.y && self.cursor_y <= win.y + win.h {
                        
                        // Bring to front logic will be handled by reordering vector logic later
                        // For now just mark active
                        clicked_idx = Some(i);

                        // Title Bar Drag Test
                        if self.cursor_y <= win.y + 25 {
                            win.is_dragging = true;
                            win.drag_offset_x = self.cursor_x - win.x;
                            win.drag_offset_y = self.cursor_y - win.y;
                        }
                        break; 
                    }
                }
                
                if let Some(idx) = clicked_idx {
                    self.active_window_idx = Some(idx);
                    // Move to end of vec to render on top?
                    // Rust borrow checker makes this annoying in a single pass.
                    // Doing naive reorder:
                    let win = self.windows.remove(idx);
                    self.windows.push(win);
                    self.active_window_idx = Some(self.windows.len() - 1);
                } else {
                    self.active_window_idx = None;
                }
            },
            4 => { // MouseUp
                self.mouse_down = false;
                for win in self.windows.iter_mut() {
                    win.is_dragging = false;
                }
            },
            5 => { // MouseMove
                self.cursor_x = event.x;
                self.cursor_y = event.y;
                
                for win in self.windows.iter_mut() {
                    if win.is_dragging {
                        win.x = self.cursor_x - win.drag_offset_x;
                        win.y = self.cursor_y - win.drag_offset_y;
                    }
                }
            },
            _ => {}
        }
    }

    fn draw(&self) {
        unsafe {
             // 1. Draw Background
            let r = ((COLOR_DESKTOP >> 24) & 0xFF) as i32;
            let g = ((COLOR_DESKTOP >> 16) & 0xFF) as i32;
            let b = ((COLOR_DESKTOP >> 8) & 0xFF) as i32;
            sys_gpu_clear(r, g, b); 

            // 2. Draw Taskbar
            let taskbar_height = 40;
            let tb_y = self.height - taskbar_height;
            sys_draw_rect(0, tb_y, self.width, taskbar_height, COLOR_GRAY);
            sys_draw_rect(0, tb_y, self.width, 2, COLOR_WHITE);
            
            // Start Button
            let btn_x = 2;
            let btn_y = tb_y + 4;
            let btn_w = 60;
            let btn_h = 32;
            
            // Pressed state if menu open
            if self.start_menu_open {
                 sys_draw_rect(btn_x, btn_y, btn_w, btn_h, COLOR_WHITE); 
                 sys_draw_rect(btn_x+1, btn_y+1, btn_w-2, btn_h-2, COLOR_GRAY);
                 sys_draw_rect(btn_x+2, btn_y+2, btn_w-4, btn_h-4, COLOR_DARK_GRAY); // Shadow inside
                 draw_text(btn_x + 12, btn_y + 10, "Start", COLOR_BLACK); // Shifted down for press effect
            } else {
                 sys_draw_rect(btn_x, btn_y, btn_w, btn_h, COLOR_WHITE); // Bevel Light
                 sys_draw_rect(btn_x+1, btn_y+1, btn_w-2, btn_h-2, COLOR_DARK_GRAY); // Bevel Shadow
                 sys_draw_rect(btn_x+1, btn_y+1, btn_w-3, btn_h-3, COLOR_GRAY); // Face
                 draw_text(btn_x + 10, btn_y + 8, "Start", COLOR_BLACK);
            }
            
            // Time Removed as per user request

            // 3. Draw Windows
            for (i, win) in self.windows.iter().enumerate() {
                let is_active = self.active_window_idx == Some(i);
                draw_window(win, is_active);
            }
            
            // 4. Draw Start Menu if open
            if self.start_menu_open {
                let menu_w = 150;
                let menu_h = 200;
                let menu_x = 2;
                let menu_y = tb_y - menu_h;
                
                sys_draw_rect(menu_x, menu_y, menu_w, menu_h, COLOR_GRAY);
                sys_draw_rect(menu_x, menu_y, menu_w, 2, COLOR_WHITE);
                sys_draw_rect(menu_x, menu_y, 2, menu_h, COLOR_WHITE);
                sys_draw_rect(menu_x + menu_w - 2, menu_y, 2, menu_h, COLOR_DARK_GRAY);
                sys_draw_rect(menu_x, menu_y + menu_h - 2, menu_w, 2, COLOR_DARK_GRAY);
                
                // Blue sidebar
                sys_draw_rect(menu_x + 3, menu_y + 3, 20, menu_h - 6, COLOR_NAVY);
                // "OS" text vertical? No, simple items.
                
                draw_text(menu_x + 30, menu_y + 10, "Programs", COLOR_BLACK);
                draw_text(menu_x + 30, menu_y + 30, "Documents", COLOR_BLACK);
                draw_text(menu_x + 30, menu_y + 50, "Settings", COLOR_BLACK);
                draw_text(menu_x + 30, menu_y + 70, "Shutdown", COLOR_BLACK);
            }

            // 5. Draw Cursor
            draw_cursor(self.cursor_x, self.cursor_y, self.mouse_down);
        }
    }
}

unsafe fn draw_window(win: &Window, is_active: bool) {
    // Shadow
    sys_draw_rect(win.x + 5, win.y + 5, win.w, win.h, 0x00_00_00_80); 
    
    // Border/Face
    sys_draw_rect(win.x, win.y, win.w, win.h, COLOR_GRAY); // Face
    sys_draw_rect(win.x, win.y, win.w, 1, COLOR_WHITE); // Top Light
    sys_draw_rect(win.x, win.y, 1, win.h, COLOR_WHITE); // Left Light
    sys_draw_rect(win.x + win.w - 1, win.y, 1, win.h, COLOR_DARK_GRAY); // Right Shadow
    sys_draw_rect(win.x, win.y + win.h - 1, win.w, 1, COLOR_DARK_GRAY); // Bottom Shadow

    // Title Bar
    let title_color = if is_active { COLOR_NAVY } else { COLOR_DARK_GRAY };
    sys_draw_rect(win.x + 3, win.y + 3, win.w - 6, 18, title_color);
    
    // Title Text
    draw_text(win.x + 6, win.y + 6, &win.title, COLOR_WHITE);
    
    // Close Button (Mock)
    sys_draw_rect(win.x + win.w - 20, win.y + 5, 14, 14, COLOR_GRAY);
    sys_draw_rect(win.x + win.w - 19, win.y + 6, 12, 12, COLOR_GRAY); // Bevel
    draw_text(win.x + win.w - 16, win.y + 6, "x", COLOR_BLACK);

    // Content Area
    let content_x = win.x + 4;
    let content_y = win.y + 24;
    let content_w = win.w - 8;
    let content_h = win.h - 28;
    sys_draw_rect(content_x, content_y, content_w, content_h, COLOR_WHITE);
    
    // Draw Context
    match win.content_type.as_str() {
        "file_manager" => {
             // Mock Files
             draw_text(content_x + 10, content_y + 10, "[DIR] bin", COLOR_BLACK);
             draw_text(content_x + 10, content_y + 30, "[DIR] usr", COLOR_BLACK);
             draw_text(content_x + 10, content_y + 50, "[DIR] home", COLOR_BLACK);
             draw_text(content_x + 10, content_y + 70, "README.md", COLOR_BLACK);
        },
        "terminal" => {
             sys_draw_rect(content_x, content_y, content_w, content_h, COLOR_BLACK);
             draw_text(content_x + 5, content_y + 5, "user@wasmix:~ $ ls", COLOR_WHITE);
             draw_text(content_x + 5, content_y + 25, "bin usr home README.md", COLOR_WHITE);
             draw_text(content_x + 5, content_y + 45, "user@wasmix:~ $ _", COLOR_WHITE);
        },
        _ => {}
    }
}

unsafe fn draw_cursor(x: i32, y: i32, down: bool) {
    // Draw a nice arrow/pointer
    let color_outline = COLOR_BLACK;
    let color_fill = COLOR_WHITE;
    
    // Simple Arrow Bitmap (Mocked by rects for now)
    // Tip at x,y
    
    // Outline
    sys_draw_rect(x, y, 1, 14, color_outline);    // Vertical left
    sys_draw_rect(x, y, 10, 1, color_outline);    // Top
    sys_draw_rect(x+1, y+1, 1, 12, color_fill);   // Fill Vert
    
    // Diagonal
    for i in 0..10 {
        sys_draw_rect(x + i, y + i, 2, 11-i, color_fill); // Solid fill body (imperfect)
        sys_draw_rect(x + i, y + i, 1, 1, color_outline); // Diagonal edge
    }
    
    // Just drawing a classic shape manually
    //      *
    //      **
    //      ***
    //      ****
    //      *****
    //      ******
    
    // Revert to simple predictable shape if complex loop fails
    
    // Arrow Vert
    sys_draw_rect(x, y, 2, 16, color_outline);
    // Arrow Diag
    // ...
    // Let's do a Crosshair/Pointer hybrid that is easy to draw with rects
    // White Circle with Black Outline?
    
    // Circle approximation (Square with cut corners)
    // 10x10
    let cx = x - 5;
    let cy = y - 5;
    
    // Crosshair lines
    sys_draw_rect(x - 8, y, 17, 1, color_black_alpha(128));
    sys_draw_rect(x, y - 8, 1, 17, color_black_alpha(128));
    
    // White Box Pointer
    sys_draw_rect(x, y, 10, 10, color_fill);
    sys_draw_rect(x, y, 10, 1, color_outline);
    sys_draw_rect(x, y, 1, 10, color_outline);
    sys_draw_rect(x+9, y, 1, 10, color_outline);
    sys_draw_rect(x, y+9, 10, 1, color_outline);
}

fn color_black_alpha(a: u8) -> i32 {
    (0x00_00_00_00 | (a as u32)) as i32
}

// Global Single State
static mut STATE: Option<DesktopState> = None;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        sys_enable_gui_mode();
        STATE = Some(DesktopState::new());
        if let Some(state) = STATE.as_ref() {
            state.draw();
        }
    }
}

#[no_mangle]
pub extern "C" fn step() {
    unsafe {
        if let Some(state) = STATE.as_mut() {
             // Poll Events
            let mut event_bytes = [0u8; 16];
            loop {
                let res = sys_poll_event(event_bytes.as_mut_ptr());
                if res == 1 {
                    let type_u32 = u32::from_le_bytes(event_bytes[0..4].try_into().unwrap());
                    let code_u32 = u32::from_le_bytes(event_bytes[4..8].try_into().unwrap());
                    let x_i32 = i32::from_le_bytes(event_bytes[8..12].try_into().unwrap());
                    let y_i32 = i32::from_le_bytes(event_bytes[12..16].try_into().unwrap());
                    
                    let event = SystemEvent {
                        event_type: type_u32,
                        code: code_u32,
                        x: x_i32,
                        y: y_i32,
                    };

                    state.update(event);
                } else {
                    break;
                }
            }
            
            state.draw();
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    init();
}
