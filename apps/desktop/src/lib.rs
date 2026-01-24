mod ui;
mod window;
mod wm;

// System Calls
extern "C" {
    fn sys_gpu_width() -> i32;
    fn sys_gpu_height() -> i32;
    fn sys_enable_gui_mode();
    fn sys_poll_event(ptr: *mut u8) -> i32;
}

static mut WM: Option<wm::WindowManager> = None;

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        sys_enable_gui_mode();
        let w = sys_gpu_width();
        let h = sys_gpu_height();
        let mut manager = wm::WindowManager::new(w, h);
        manager.init();
        
        *std::ptr::addr_of_mut!(WM) = Some(manager);
        if let Some(wm) = (*std::ptr::addr_of_mut!(WM)).as_ref() {
            wm.draw();
        }
    }
}

#[no_mangle]
pub extern "C" fn step() {
    unsafe {
        if let Some(wm) = (*std::ptr::addr_of_mut!(WM)).as_mut() {
            // Poll Events
            let mut event_bytes = [0u8; 16];
            loop {
                let res = sys_poll_event(event_bytes.as_mut_ptr());
                if res == 1 {
                    let type_u32 = u32::from_le_bytes(event_bytes[0..4].try_into().unwrap());
                    let code = u32::from_le_bytes(event_bytes[4..8].try_into().unwrap());
                    let x = i32::from_le_bytes(event_bytes[8..12].try_into().unwrap());
                    let y = i32::from_le_bytes(event_bytes[12..16].try_into().unwrap());
                    
                    match type_u32 {
                        1 => wm.handle_key(code),
                        3 => wm.handle_mouse_down(x, y),
                        4 => wm.handle_mouse_up(),
                        5 => wm.handle_mouse_move(x, y),
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            wm.draw();
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    init();
}
