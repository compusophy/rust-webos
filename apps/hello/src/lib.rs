#[link(wasm_import_module = "env")]
extern "C" {
    fn sys_print(ptr: *const u8, len: usize);
}

#[no_mangle]
pub extern "C" fn _start() {
    let msg = "hello from wasm executable!";
    unsafe {
        sys_print(msg.as_ptr(), msg.len());
    }
}
