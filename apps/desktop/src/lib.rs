use std::ffi::CString;

// System Calls
extern "C" {
    fn sys_print(ptr: *const u8, len: usize);
    fn sys_gpu_width() -> i32;
    fn sys_gpu_height() -> i32;
    fn sys_gpu_clear(r: i32, g: i32, b: i32);
    fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    fn sys_enable_gui_mode();
    fn sys_poll_event(ptr: *mut u8) -> i32;
}

// ... Structs (SystemEvent, Window, DesktopState) same as before ... 
// Copied from previous logic

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

struct Window {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: String,
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
    initialized: bool,
}

impl DesktopState {
    fn new() -> Self {
        let width = unsafe { sys_gpu_width() };
        let height = unsafe { sys_gpu_height() };
        
        let mut windows = Vec::new();
        windows.push(Window {
            x: 50, y: 50, w: 300, h: 200,
            title: "File Manager".to_string(),
            is_dragging: false, drag_offset_x: 0, drag_offset_y: 0,
        });

        windows.push(Window {
            x: 100, y: 100, w: 300, h: 200,
            title: "Terminal".to_string(),
            is_dragging: false, drag_offset_x: 0, drag_offset_y: 0,
        });

        Self {
            width,
            height,
            windows,
            cursor_x: width / 2,
            cursor_y: height / 2,
            mouse_down: false,
            initialized: false, 
        }
    }
    
    // ... update() and draw() same as before ...
     fn update(&mut self, event: SystemEvent) {
        match event.event_type {
            3 => { // MouseDown
                self.mouse_down = true;
                // Check collisions (reverse order for z-index)
                for win in self.windows.iter_mut().rev() {
                    // Title Bar Hit Test
                    if self.cursor_x >= win.x && self.cursor_x <= win.x + win.w &&
                       self.cursor_y >= win.y && self.cursor_y <= win.y + 25 {
                        win.is_dragging = true;
                        win.drag_offset_x = self.cursor_x - win.x;
                        win.drag_offset_y = self.cursor_y - win.y;
                        break; // Only pick one
                    }
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
            sys_draw_rect(0, self.height - taskbar_height, self.width, taskbar_height, COLOR_GRAY);
            
            // Start Button
            sys_draw_rect(2, self.height - taskbar_height + 2, 60, taskbar_height - 4, COLOR_GRAY);
            sys_draw_rect(2, self.height - taskbar_height + 2, 60, 2, COLOR_WHITE);
            sys_draw_rect(2, self.height - taskbar_height + 2, 2, taskbar_height - 4, COLOR_WHITE);
            sys_draw_rect(60, self.height - taskbar_height + 2, 2, taskbar_height - 4, COLOR_DARK_GRAY);
            sys_draw_rect(2, self.height - 2, 60, 2, COLOR_DARK_GRAY);

            // 3. Draw Windows
            for win in &self.windows {
                draw_window(win);
            }
            
            // 4. Draw Cursor
            draw_cursor(self.cursor_x, self.cursor_y, self.mouse_down);
        }
    }
}

unsafe fn draw_window(win: &Window) {
    sys_draw_rect(win.x + 5, win.y + 5, win.w, win.h, 0x00_00_00_80); 
    sys_draw_rect(win.x, win.y, win.w, win.h, COLOR_GRAY);
    let title_color = if win.is_dragging { COLOR_BLUE } else { 0x00_00_40_FFu32 as i32 };
    sys_draw_rect(win.x + 3, win.y + 3, win.w - 6, 20, title_color);
    sys_draw_rect(win.x + 3, win.y + 26, win.w - 6, win.h - 30, COLOR_WHITE);
}

unsafe fn draw_cursor(x: i32, y: i32, down: bool) {
    let color = if down { COLOR_BLACK } else { COLOR_WHITE };
    sys_draw_rect(x, y, 2, 12, color);
    sys_draw_rect(x, y, 8, 2, color);
    sys_draw_rect(x+1, y+1, 2, 8, color);
    sys_draw_rect(x+1, y+1, 6, 2, color);
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
            // Loop until queue empty? 
            // Or just one per step?
            // Better to process all pending events to track mouse smoothly
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
    // Legacy fallback: if host calls _start expecting blocking behavior
    // We can just call init.
    // If host doesn't support step, this will exit and state is lost (if using stack)
    // But we used static mut STATE, so it persists?
    // But who calls update?
    init();
}
