
extern "C" {
    pub fn sys_draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32);
    pub fn sys_draw_text(ptr: *const u8, len: i32, x: i32, y: i32, color: i32);
    #[allow(dead_code)]
    pub fn sys_gpu_width() -> i32;
    #[allow(dead_code)]
    pub fn sys_gpu_height() -> i32;
    pub fn sys_gpu_clear(r: i32, g: i32, b: i32);
    pub fn sys_poll_event(ptr: *mut u8) -> i32;
    pub fn sys_exec(cmd_ptr: *const u8, cmd_len: i32, out_ptr: *mut u8, out_len: i32) -> i32;
    pub fn sys_fs_getcwd(out_ptr: *mut u8, out_len: i32) -> i32;
    pub fn sys_enable_gui_mode();
    #[allow(dead_code)]
    pub fn sys_time() -> i32;
}

pub fn enable_gui_mode() {
    unsafe { sys_enable_gui_mode(); }
}

pub fn clear_screen() {
    unsafe { sys_gpu_clear(0, 0, 0); }
}

// ...

pub fn time() -> f64 {
    unsafe {
        sys_time() as f64
    }
}

pub fn draw_rect(x: i32, y: i32, w: i32, h: i32, color: i32) {
    unsafe {
        sys_draw_rect(x, y, w, h, color);
    }
}

pub fn draw_text(x: i32, y: i32, text: &str, color: i32) {
    unsafe {
        sys_draw_text(text.as_ptr(), text.len() as i32, x, y, color);
    }
}

pub fn exec(cmd: &str) -> String {
    let mut out_buf = [0u8; 8192]; // Larger buffer for shell output
    let res = unsafe {
        sys_exec(cmd.as_ptr(), cmd.len() as i32, out_buf.as_mut_ptr(), 8192)
    };
    
    if res >= 0 {
        let s = std::str::from_utf8(&out_buf[0..res as usize]).unwrap_or("");
        s.to_string()
    } else {
        format!("Error executing command: {}", cmd)
    }
}

pub fn getcwd() -> String {
    let mut out_buf = [0u8; 1024];
    let res = unsafe {
        sys_fs_getcwd(out_buf.as_mut_ptr(), 1024)
    };
    
    if res >= 0 {
        std::str::from_utf8(&out_buf[0..res as usize]).unwrap_or("~").to_string()
    } else {
        "~".to_string()
    }
}


