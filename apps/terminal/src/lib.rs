mod term;
mod ui;

static mut TERMINAL: Option<term::Terminal> = None;
static mut INPUT_BUFFER: String = String::new();
static mut HISTORY: Vec<String> = Vec::new();
static mut HISTORY_INDEX: Option<usize> = None;

#[no_mangle]
pub extern "C" fn init() {
    ui::enable_gui_mode();
    ui::clear_screen();
    
    // Initialize Terminal (Standard VGA size roughly, fits 640x480)
    // 640 / 8 = 80 cols
    // 480 / 16 = 30 rows (Full Height)
    // ACTUAL GPU IS 512x512.
    // 512 / 8 = 64 Cols
    // 512 / 16 = 32 Rows
    let mut term = term::Terminal::new(64, 32);
    
    // Write Initial Prompt
    write_prompt(&mut term);
    
    unsafe {
        TERMINAL = Some(term);
    }
}

fn write_prompt(term: &mut term::Terminal) {
    // "user" -> Orange
    term.set_fg_color(0xFF_A5_00_FFu32 as i32);
    term.write_str("user");
    
    // "@" -> White
    term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
    term.write_str("@");
    
    // "wasmix" -> Green
    term.set_fg_color(0x00_FF_00_FFu32 as i32);
    term.write_str("wasmix");
    
    // ":path $ " -> White
    let path = ui::getcwd();
    term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
    term.write_str(":");
    term.write_str(&path);
    term.write_str("$ ");
}

#[no_mangle]
pub extern "C" fn step() {
    ui::enable_gui_mode();

    unsafe {
        if let Some(term) = &mut *std::ptr::addr_of_mut!(TERMINAL) {
            // Render at 4, 0. Match kernel.
            term.render(4, 0);
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
        let term = match (*std::ptr::addr_of_mut!(TERMINAL)).as_mut() {
            Some(t) => t,
            None => return,
        };

        match code {
            10 => { // Enter
                let cmd = (*std::ptr::addr_of_mut!(INPUT_BUFFER)).trim().to_string();
                
                // New Line handled by execute or below
                term.write_char('\n'); 
                
                if !cmd.is_empty() {
                    (*std::ptr::addr_of_mut!(HISTORY)).push(cmd.clone());
                    *std::ptr::addr_of_mut!(HISTORY_INDEX) = None;
                    
                    if cmd == "clear" {
                        term.reset();
                        // Repaint prompt
                        write_prompt(term);
                        (*std::ptr::addr_of_mut!(INPUT_BUFFER)).clear();
                        return;
                    } else {
                        // Exec
                        let output = ui::exec(&cmd);
                        
                        if !output.is_empty() {
                            // Output is White
                            term.set_fg_color(0xFF_FF_FF_FFu32 as i32);
                            term.write_str(&output);
                            if !output.ends_with('\n') {
                                term.write_char('\n');
                            }
                        }
                    }
                }
                
                (*std::ptr::addr_of_mut!(INPUT_BUFFER)).clear();
                write_prompt(term);
            },
            8 => { // Backspace
                let input = &mut *std::ptr::addr_of_mut!(INPUT_BUFFER);
                if !input.is_empty() {
                    input.pop();
                    term.write_char('\x08');
                }
            },
            _ => {
                if let Some(c) = std::char::from_u32(code) {
                    if c.is_ascii_graphic() || c == ' ' {
                        (*std::ptr::addr_of_mut!(INPUT_BUFFER)).push(c);
                        // Input stays Colored (Green)
                        term.set_fg_color(0x00_FF_00_FFu32 as i32); 
                        term.write_char(c);
                    }
                }
            }
        }
    }
}
