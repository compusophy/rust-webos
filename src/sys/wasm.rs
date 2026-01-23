use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use wasmi::{Engine, Linker, Module, Store, Caller, Instance};

pub struct WasmRuntime {
    engine: Engine,
    term: Rc<RefCell<crate::term::Terminal>>,
    gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
    gui_mode: Rc<RefCell<bool>>,
    events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
    fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
    
    active_process: RefCell<Option<ActiveProcess>>,
}

struct ActiveProcess {
    store: Store<WasmContext>,
    instance: Instance,
}

pub struct WasmContext {
    // We hold Rcs here too? Or weak?
    // The Host Functions need access to the data.
    // If we use Rcs, we can clone them into the closure?
    // WasmContext stores "Host State".
    // Actually, Host functions receive `Caller<T>`. `T` is WasmContext.
    // So WasmContext should hold the Rcs.
    pub term: Rc<RefCell<crate::term::Terminal>>,
    pub gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
    pub gui_mode: Rc<RefCell<bool>>,
    pub events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
    pub fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
}

impl WasmRuntime {
    pub fn new(
        term: Rc<RefCell<crate::term::Terminal>>,
        gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
        gui_mode: Rc<RefCell<bool>>,
        events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
        fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
    ) -> Self {
        let engine = Engine::default();
        Self {
            engine,
            term,
            gpu,
            gui_mode,
            events,
            fs,
            active_process: RefCell::new(None),
        }
    }

    pub fn load(&self, wasm_bytes: &[u8]) -> Result<(), String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("failed to create module: {}", e))?;

        let ctx = WasmContext {
            term: self.term.clone(),
            gpu: self.gpu.clone(),
            gui_mode: self.gui_mode.clone(),
            events: self.events.clone(),
            fs: self.fs.clone(),
        };

        let mut store = Store::new(&self.engine, ctx);
        let mut linker = Linker::new(&self.engine);
        
        // --- Host Functions ---

        linker.func_wrap("env", "sys_print", |mut caller: Caller<WasmContext>, ptr: i32, len: i32| {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut buffer = vec![0u8; len as usize];
                if extern_mem.read(&caller, ptr as usize, &mut buffer).is_ok() {
                    if let Ok(msg) = String::from_utf8(buffer) {
                        let term = caller.data().term.clone(); // Clone Rc
                        let mut term = term.borrow_mut();
                        term.write_str(&msg);
                        term.write_char('\n');
                    }
                }
            }
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_width", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.borrow().width as i32
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_height", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.borrow().height as i32
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_clear", |mut caller: Caller<WasmContext>, r: i32, g: i32, b: i32| {
            caller.data().gpu.borrow_mut().clear(r as u8, g as u8, b as u8);
        }).unwrap();

        linker.func_wrap("env", "sys_draw_rect", |mut caller: Caller<WasmContext>, x: i32, y: i32, w: i32, h: i32, color: i32| {
             caller.data().gpu.borrow_mut().fill_rect(x as u32, y as u32, w as u32, h as u32, color as u32);
        }).unwrap();

        linker.func_wrap("env", "sys_enable_gui_mode", |mut caller: Caller<WasmContext>| {
            *caller.data().gui_mode.borrow_mut() = true;
        }).unwrap();

        linker.func_wrap("env", "sys_poll_event", |mut caller: Caller<WasmContext>, ptr: i32| -> i32 {
             let mut events_guard = caller.data().events.borrow_mut();
             if let Some(event) = events_guard.pop_front() {
                 if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                     let type_val = event.event_type as u32;
                     let code_val = event.code;
                     let x_val = event.x as u32; 
                     let y_val = event.y as u32;
                     
                     let bytes = [
                         type_val.to_le_bytes(),
                         code_val.to_le_bytes(),
                         x_val.to_le_bytes(),
                         y_val.to_le_bytes(),
                     ].concat(); 
                     
                     if extern_mem.write(&mut caller, ptr as usize, &bytes).is_ok() {
                         return 1;
                     }
                 }
             }
             0
        }).unwrap();

        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("failed to instantiate: {}", e))?
            .start(&mut store)
            .map_err(|e| format!("failed to start: {}", e))?;

        // LOOK FOR init() function
        if let Ok(init_func) = instance.get_typed_func::<(), ()>(&store, "init") {
             init_func.call(&mut store, ())
                .map_err(|e| format!("init error: {}", e))?;
        } else if let Ok(start_func) = instance.get_typed_func::<(), ()>(&store, "_start") {
             // Fallback for non-async apps (hello, math) -> Run once and Drop?
             // If we run `hello`, it prints and exits.
             // If we run `desktop` with `_start` loop, it blocks.
             // We need `desktop` to export `init` and `step`.
             // For legacy compatibility, if `init` missing, run `_start`.
             start_func.call(&mut store, ())
                .map_err(|e| format!("start error: {}", e))?;
             // If it returns, we are done. Don't save process unless we want 'step'?
             // Non-async apps don't have step.
             return Ok(());
        }

        // Save process if it has a `step` function
        if let Ok(_) = instance.get_typed_func::<(), ()>(&store, "step") {
            *self.active_process.borrow_mut() = Some(ActiveProcess {
                store,
                instance,
            });
        }
        
        Ok(())
    }

    pub fn tick(&self) {
        // We need to borrow mutably from RefCell, but self is immutable ref?
        // `active_process` is RefCell.
        // We can interact with store mutably.
        
        let mut process_guard = self.active_process.borrow_mut();
        if let Some(process) = process_guard.as_mut() {
             if let Ok(step_func) = process.instance.get_typed_func::<(), ()>(&process.store, "step") {
                 let res = step_func.call(&mut process.store, ());
                 if let Err(e) = res {
                     // If error, kill process
                     web_sys::console::log_1(&format!("Process crashed: {}", e).into());
                     *process_guard = None;
                     // Disable GUI mode?
                     *self.gui_mode.borrow_mut() = false;
                 }
             }
        }
    }
}
