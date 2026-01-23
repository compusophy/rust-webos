use wasmi::{Engine, Linker, Module, Store, Caller};

pub struct WasmRuntime {
    engine: Engine,
}

pub struct WasmContext<'a> {
    pub term: &'a mut crate::term::Terminal,
    pub gpu: &'a mut crate::hw::gpu::Gpu,
    pub gui_mode: &'a mut bool,
}

impl WasmRuntime {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self {
            engine,
        }
    }

    pub fn run(&self, wasm_bytes: &[u8], term: &mut crate::term::Terminal, gpu: &mut crate::hw::gpu::Gpu, gui_mode: &mut bool) -> Result<(), String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("failed to create module: {}", e))?;

        // Create context wrapper
        let ctx = WasmContext { term, gpu, gui_mode };
        
        // Store holds the context (WasmContext)
        let mut store = Store::new(&self.engine, ctx);
        
        // Linker needs to match the Store's data type
        let mut linker = Linker::new(&self.engine);
        
        // Define HOST imports
        
        // env.sys_print(ptr, len)
        linker.func_wrap("env", "sys_print", |mut caller: Caller<WasmContext>, ptr: i32, len: i32| {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut buffer = vec![0u8; len as usize];
                if extern_mem.read(&caller, ptr as usize, &mut buffer).is_ok() {
                    if let Ok(msg) = String::from_utf8(buffer) {
                        let term = &mut caller.data_mut().term;
                        term.write_str(&msg);
                        term.write_char('\n');
                    }
                }
            }
        }).unwrap();

        // env.sys_gpu_width() -> i32
        linker.func_wrap("env", "sys_gpu_width", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.width as i32
        }).unwrap();

        // env.sys_gpu_height() -> i32
        linker.func_wrap("env", "sys_gpu_height", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.height as i32
        }).unwrap();

        // env.sys_gpu_clear(r, g, b)
        linker.func_wrap("env", "sys_gpu_clear", |mut caller: Caller<WasmContext>, r: i32, g: i32, b: i32| {
            caller.data_mut().gpu.clear(r as u8, g as u8, b as u8);
        }).unwrap();

        // env.sys_draw_rect(x, y, w, h, color)
        linker.func_wrap("env", "sys_draw_rect", |mut caller: Caller<WasmContext>, x: i32, y: i32, w: i32, h: i32, color: i32| {
            caller.data_mut().gpu.fill_rect(x as u32, y as u32, w as u32, h as u32, color as u32);
        }).unwrap();

        // env.sys_enable_gui_mode()
        linker.func_wrap("env", "sys_enable_gui_mode", |mut caller: Caller<WasmContext>| {
            *caller.data_mut().gui_mode = true;
        }).unwrap();

        
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("failed to instantiate: {}", e))?
            .start(&mut store)
            .map_err(|e| format!("failed to start: {}", e))?;

        // Find _start function (setup by default for no_std binaries usually, or we need to export it)
        if let Ok(start_func) = instance.get_typed_func::<(), ()>(&store, "_start") {
             start_func.call(&mut store, ())
                .map_err(|e| format!("runtime error: {}", e))?;
        } else {
             return Err("no _start entry point found".to_string());
        }

        Ok(())
    }
}
