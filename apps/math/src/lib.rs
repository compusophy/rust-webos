#[link(wasm_import_module = "env")]
extern "C" {
    fn sys_print(ptr: *const u8, len: usize);
}

#[no_mangle]
pub extern "C" fn _start() {
    let a = 10;
    let b = 32;
    let sum = a + b;
    let product = a * b;
    
    // We can't use format! easily without std or a formatting library in no_std,
    // but for simplicity in this demo, we'll construct a simple string buffer or import a mini formatter.
    // Or just simple hardcoded logic string for now to prove it works, or do manual int to string.
    
    let _msg = "math check: 10 + 32 = 42"; // Hardcoded for SAFETY/SPEED if no allocationster
    // Let's actually compute it to prove it's real code running?
    // Without `alloc` or `std`, string formatting is annoying.
    // Let's just print static strings based on the result to PROVE calculation happened.
    
    if sum == 42 {
        let success = "addition success: 10 + 32 = 42";
        unsafe { sys_print(success.as_ptr(), success.len()); }
    } else {
        let fail = "addition failed: cpu error";
        unsafe { sys_print(fail.as_ptr(), fail.len()); }
    }
    
    if product == 320 {
        let p_msg = "multiplication success: 10 * 32 = 320";
        unsafe { sys_print(p_msg.as_ptr(), p_msg.len()); }
    }
}
