use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use wasmi::{Engine, Linker, Module, Store, Caller, Instance, AsContextMut};

#[derive(Clone)]
pub struct WasmRuntime {
    engine: Engine,
    term: Rc<RefCell<crate::term::Terminal>>,
    gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
    gui_mode: Rc<RefCell<bool>>,
    events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
    fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
    should_reset: Rc<RefCell<bool>>,
    shell: Rc<RefCell<crate::sys::shell::Shell>>,
    
    active_process: Rc<RefCell<Option<ActiveProcess>>>, 
    next_process: Rc<RefCell<Option<ActiveProcess>>>,
}

pub struct ActiveProcess {
    pub store: Store<WasmContext>,
    pub instance: Instance,
}

pub struct WasmContext {
    pub term: Rc<RefCell<crate::term::Terminal>>,
    pub gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
    pub gui_mode: Rc<RefCell<bool>>,
    pub events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
    pub fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
    pub should_reset: Rc<RefCell<bool>>,
    pub shell: Rc<RefCell<crate::sys::shell::Shell>>,
    // Add access to global process slot for exec/replacement
    pub active_process: Rc<RefCell<Option<ActiveProcess>>>,
    pub next_process: Rc<RefCell<Option<ActiveProcess>>>,
}

impl WasmRuntime {
    pub fn new(
        term: Rc<RefCell<crate::term::Terminal>>,
        gpu: Rc<RefCell<crate::hw::gpu::Gpu>>,
        gui_mode: Rc<RefCell<bool>>,
        events: Rc<RefCell<VecDeque<crate::kernel::SystemEvent>>>,
        fs: Rc<RefCell<crate::sys::fs::FileSystem>>,
        should_reset: Rc<RefCell<bool>>,
        shell: Rc<RefCell<crate::sys::shell::Shell>>,
    ) -> Self {
        let engine = Engine::default();
        Self {
            engine,
            term,
            gpu,
            gui_mode,
            events,
            fs,
            should_reset,
            shell,
            active_process: Rc::new(RefCell::new(None)),
            next_process: Rc::new(RefCell::new(None)),
        }
    }

    fn setup_linker(engine: &Engine, output_buffer: std::sync::Arc<std::sync::Mutex<String>>) -> Linker<WasmContext> {
        let mut linker = Linker::new(engine);
        
        let output_clone = output_buffer.clone();
        
        // ... (sys_print, sys_fs_list, etc unchanged) ...
        linker.func_wrap("env", "sys_print", move |caller: Caller<WasmContext>, ptr: i32, len: i32| {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut buffer = vec![0u8; len as usize];
                if extern_mem.read(&caller, ptr as usize, &mut buffer).is_ok() {
                    if let Ok(msg) = String::from_utf8(buffer) {
                        // Write to local capture
                        if let Ok(mut out) = output_clone.lock() {
                             out.push_str(&msg);
                             out.push('\n');
                        }

                        if let Ok(mut term_guard) = caller.data().term.try_borrow_mut() {
                            term_guard.write_str(&msg);
                            term_guard.write_char('\n');
                        } else {
                            // Recovery
                            if let Ok(mut gpu) = caller.data().gpu.try_borrow_mut() {
                                let mut draw_x = 10;
                                let draw_y = 480; 
                                let alert = format!("kernel i/o: {}", msg);
                                for c in alert.chars() {
                                     crate::gfx::font::draw_char(&mut gpu, draw_x, draw_y, c, 0xFF_00_00_FF);
                                     draw_x += 8;
                                }
                            }
                        }
                    }
                }
            }
        }).unwrap();

        linker.func_wrap("env", "sys_fs_list", |mut caller: Caller<WasmContext>, path_ptr: i32, path_len: i32, out_ptr: i32, out_len: i32| -> i32 {
             if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut path_buf = vec![0u8; path_len as usize];
                if extern_mem.read(&caller, path_ptr as usize, &mut path_buf).is_ok() {
                    if let Ok(path_str) = String::from_utf8(path_buf) {
                        let output_data = {
                            let fs = caller.data().fs.borrow();
                            let path_parts: Vec<String> = path_str.split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                            let target_node = if path_str == "/" { Some(&fs.root) } else { fs.resolve_dir(&path_parts) };
                            
                            if let Some(node) = target_node {
                                 if let crate::sys::fs::NodeType::Directory = node.node_type {
                                     let mut output = String::new();
                                     if !path_parts.is_empty() { output.push_str("D:..\n"); }
                                     let mut entries: Vec<_> = node.children.iter().collect();
                                     entries.sort_by_key(|(k,_)| *k);
                                     for (name, child) in entries {
                                         let prefix = match child.node_type {
                                             crate::sys::fs::NodeType::Directory => "D",
                                             crate::sys::fs::NodeType::File => "F",
                                         };
                                         output.push_str(&format!("{}:{}\n", prefix, name));
                                     }
                                     Some(output)
                                 } else { None }
                            } else { None }
                        };

                        if let Some(output) = output_data {
                             let bytes = output.as_bytes();
                             let write_len = bytes.len().min(out_len as usize);
                             extern_mem.write(&mut caller.as_context_mut(), out_ptr as usize, &bytes[0..write_len]).ok();
                             return write_len as i32;
                        }
                    }
                }
             }
             -1
        }).unwrap();

        linker.func_wrap("env", "sys_fs_getcwd", |mut caller: Caller<WasmContext>, out_ptr: i32, out_len: i32| -> i32 {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let path_str = {
                    let fs = caller.data().fs.borrow();
                    if fs.current_path.is_empty() { "~".to_string() } else {
                        let mut p = String::from("/");
                        for part in &fs.current_path { p.push_str(part); p.push('/'); }
                        if p.len() > 1 { p.pop(); }
                        p
                    }
                };
                let bytes = path_str.as_bytes();
                let write_len = bytes.len().min(out_len as usize);
                extern_mem.write(&mut caller.as_context_mut(), out_ptr as usize, &bytes[0..write_len]).ok();
                return write_len as i32;
            }
            -1
        }).unwrap();

        // RECURSIVE SYS_EXEC
        linker.func_wrap("env", "sys_exec", move |mut caller: Caller<WasmContext>, cmd_ptr: i32, cmd_len: i32, out_ptr: i32, out_len: i32| -> i32 {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut cmd_buf = vec![0u8; cmd_len as usize];
                if extern_mem.read(&caller, cmd_ptr as usize, &mut cmd_buf).is_ok() {
                    if let Ok(cmd_str) = String::from_utf8(cmd_buf) {
                        
                        // Parse command locally to check for empty input
                        let cmd_str_trim = cmd_str.trim();
                        if cmd_str_trim.is_empty() { return -1; }
                        
                        let output = {
                            // Construct transient runtime ISOLATED from parent engine but SHARING process slot
                            let runtime = WasmRuntime {
                                engine: Engine::default(), // New Engine to avoid deadlock
                                term: caller.data().term.clone(),
                                gpu: caller.data().gpu.clone(),
                                gui_mode: caller.data().gui_mode.clone(),
                                events: caller.data().events.clone(),
                                fs: caller.data().fs.clone(),
                                should_reset: caller.data().should_reset.clone(),
                                shell: caller.data().shell.clone(),
                                // VITAL: Share the global active_process slot with the kernel
                                active_process: caller.data().active_process.clone(), 
                                next_process: caller.data().next_process.clone(),
                            };

                            // Use a transient Shell to avoid RefCell Double Borrow Panic
                            // The global kernel shell might be active (e.g. in run_one_command -> exec -> sys_exec)
                            let mut shell = crate::sys::shell::Shell::new(); 
                            
                            let fs_rc = caller.data().fs.clone();
                            let events_rc = caller.data().events.clone();
                            
                            // execute_string calls load(), which updates active_process if successful
                            // CRITICAL: Pass the Rc, not a borrow, to avoid double-borrow panics in nested sys_exec
                            let (out_str, reboot) = shell.execute_string_rc(cmd_str_trim, &fs_rc, Some(&runtime), &events_rc, 0, 0.0);
                            
                            if reboot {
                                *caller.data().should_reset.borrow_mut() = true;
                            }
                            
                            out_str
                        };

                        let bytes = output.as_bytes();
                        let write_len = bytes.len().min(out_len as usize);
                        extern_mem.write(&mut caller.as_context_mut(), out_ptr as usize, &bytes[0..write_len]).ok();
                        return write_len as i32;
                    }
                }
            }
            -1
        }).unwrap();

        // ... (rest unchanged) ...
        linker.func_wrap("env", "sys_reset", |caller: Caller<WasmContext>| {
             if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.remove_item("wasmix_fs_local");
                }
            }
            *caller.data().should_reset.borrow_mut() = true;
        }).unwrap();

        linker.func_wrap("env", "sys_restart", |caller: Caller<WasmContext>| {
            *caller.data().should_reset.borrow_mut() = true;
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_width", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.borrow().width as i32
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_height", |caller: Caller<WasmContext>| -> i32 {
            caller.data().gpu.borrow().height as i32
        }).unwrap();

        linker.func_wrap("env", "sys_gpu_clear", |caller: Caller<WasmContext>, r: i32, g: i32, b: i32| {
            caller.data().gpu.borrow_mut().clear(r as u8, g as u8, b as u8);
        }).unwrap();

        linker.func_wrap("env", "sys_draw_rect", |caller: Caller<WasmContext>, x: i32, y: i32, w: i32, h: i32, color: i32| {
             caller.data().gpu.borrow_mut().fill_rect(x, y, w, h, color as u32);
        }).unwrap();

        linker.func_wrap("env", "sys_draw_text", |caller: Caller<WasmContext>, ptr: i32, len: i32, x: i32, y: i32, color: i32| {
            if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut buffer = vec![0u8; len as usize];
                if extern_mem.read(&caller, ptr as usize, &mut buffer).is_ok() {
                    if let Ok(msg) = String::from_utf8(buffer) {
                        let mut gpu = caller.data().gpu.borrow_mut();
                        let mut draw_x = x;
                        let draw_y = y;
                        for c in msg.chars() {
                             crate::gfx::font::draw_char(&mut gpu, draw_x, draw_y, c, color as u32);
                             draw_x += 8;
                        }
                    }
                }
            }
        }).unwrap();

        linker.func_wrap("env", "sys_enable_gui_mode", |caller: Caller<WasmContext>| {
            *caller.data().gui_mode.borrow_mut() = true;
        }).unwrap();

        linker.func_wrap("env", "sys_poll_event", |mut caller: Caller<WasmContext>, ptr: i32| -> i32 {
             let events_rc = caller.data().events.clone();
             let mut events_guard = events_rc.borrow_mut();
             if let Some(event) = events_guard.pop_front() {
                 if let Some(extern_mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                     let type_val = event.event_type as u32;
                     let code_val = event.code;
                     let x_val = event.x as u32; 
                     let y_val = event.y as u32;
                     let bytes = [type_val.to_le_bytes(), code_val.to_le_bytes(), x_val.to_le_bytes(), y_val.to_le_bytes()].concat(); 
                     if extern_mem.write(&mut caller, ptr as usize, &bytes).is_ok() { return 1; }
                 }
             }
             0
        }).unwrap();

        linker.func_wrap("env", "sys_time", |_caller: Caller<WasmContext>| -> i32 {
            if let Some(window) = web_sys::window() {
                if let Some(perf) = window.performance() { return perf.now() as i32; }
            }
            0
        }).unwrap();
        
        linker
    }

    pub fn load(&self, wasm_bytes: &[u8]) -> Result<String, String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("failed to create module: {}", e))?;
        
        let output_buffer = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let ctx = WasmContext {
            term: self.term.clone(),
            gpu: self.gpu.clone(),
            gui_mode: self.gui_mode.clone(),
            events: self.events.clone(),
            fs: self.fs.clone(),
            should_reset: self.should_reset.clone(),
            shell: self.shell.clone(),
            active_process: self.active_process.clone(),
            next_process: self.next_process.clone(),
        };

        let mut store = Store::new(&self.engine, ctx);
        let linker = Self::setup_linker(&self.engine, output_buffer.clone());

        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format!("failed to instantiate: {}", e))?
            .start(&mut store)
            .map_err(|e| format!("failed to start: {}", e))?;

        if let Ok(init_func) = instance.get_typed_func::<(), ()>(&store, "init") {
             init_func.call(&mut store, ())
                .map_err(|e| format!("init error: {}", e))?;
        } else if let Ok(start_func) = instance.get_typed_func::<(), ()>(&store, "_start") {
             start_func.call(&mut store, ())
                .map_err(|e| format!("start error: {}", e))?;
        }

        if let Ok(_) = instance.get_typed_func::<(), ()>(&store, "step") {
            
            // Try explicit borrow first (works for Boot)
            if let Ok(mut guard) = self.active_process.try_borrow_mut() {
                *guard = Some(ActiveProcess {
                    store,
                    instance,
                });
            } else {
                // If active_process is busy (e.g. inside tick -> sys_exec), schedule switch
                if let Ok(mut guard) = self.next_process.try_borrow_mut() {
                    *guard = Some(ActiveProcess {
                        store,
                        instance,
                    });
                }
            }
        }
        
        let res = output_buffer.lock().unwrap().clone();
        Ok(res)
    }

    pub fn load_from_path(&self, path: &str) -> Result<String, String> {
        let content = {
            let fs = self.fs.borrow();
            let node = if let Some(node) = fs.resolve_dir(&path.split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<_>>()) {
                 Some(node)
            } else {
                 if let Some(bin) = fs.root.children.get("bin") {
                     if let Some(node) = bin.children.get(path) {
                         Some(node)
                     } else if path.starts_with("/bin/") {
                          if let Some(node) = bin.children.get(&path[5..]) { Some(node) } else { None }
                     } else { None }
                 } else { None }
            };

            if let Some(node) = node {
                if let crate::sys::fs::NodeType::File = node.node_type {
                    Some(node.content.clone())
                } else { None }
            } else { None }
        }; 

        if let Some(bytes) = content {
             self.load(&bytes)
        } else {
             Err("file not found".to_string())
        }
    }

    pub fn tick(&self) {
        {
            let mut process_guard = self.active_process.borrow_mut();
            if let Some(process) = process_guard.as_mut() {
                 if let Ok(step_func) = process.instance.get_typed_func::<(), ()>(&process.store, "step") {
                     let res = step_func.call(&mut process.store, ());
                     if let Err(e) = res {
                         web_sys::console::log_1(&format!("process crashed: {}", e).into());
                         *process_guard = None;
                         *self.gui_mode.borrow_mut() = false;
                     }
                 }
            }
        } // Drop active lock

        // Apply pending Process Switch
        if let Ok(mut next_guard) = self.next_process.try_borrow_mut() {
             if next_guard.is_some() {
                 *self.active_process.borrow_mut() = next_guard.take();
             }
        }
    }
}
