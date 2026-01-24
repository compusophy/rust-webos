mod term;
mod ui;
mod shell;

use shell::Shell;

static mut SHELL: Option<Shell> = None;

#[no_mangle]
pub extern "C" fn init() {
    ui::enable_gui_mode();
    ui::clear_screen();
    
    // Initialize Shell (Standard VGA size roughly, fits 640x480)
    // 640 / 8 = 80 cols
    // 480 / 16 = 30 rows (Full Height)
    // ACTUAL GPU IS 512x512.
    // 512 / 8 = 64 Cols
    // 512 / 16 = 32 Rows
    let shell = Shell::new(64, 32);
    
    unsafe {
        SHELL = Some(shell);
    }
}

#[no_mangle]
pub extern "C" fn step() {
    ui::enable_gui_mode();

    unsafe {
        if let Some(shell) = &mut *std::ptr::addr_of_mut!(SHELL) {
            // Render at 4, 0. Match kernel.
            shell.draw(4, 0);
        }
    }

    // Poll Events
    let mut buf = [0u8; 16]; 
    loop {
        let res = unsafe { ui::sys_poll_event(buf.as_mut_ptr()) };
        if res == 0 { break; }
        
        let type_val = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let code_val = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        
        if type_val == 1 { // KeyDown
            handle_key(code_val);
        }
    }
}

fn handle_key(code: u32) {
    unsafe {
        if let Some(shell) = (*std::ptr::addr_of_mut!(SHELL)).as_mut() {
            shell.on_key(code);
        }
    }
}

