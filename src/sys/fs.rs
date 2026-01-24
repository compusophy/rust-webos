use std::collections::HashMap;
use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub size: usize, // Bytes
    pub children: HashMap<String, Node>, // For directories
    pub content: Vec<u8>, // For files
}

impl Node {
    pub fn new_dir(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node_type: NodeType::Directory,
            size: 0,
            children: HashMap::new(),
            content: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn new_file(name: &str, content: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            node_type: NodeType::File,
            size: content.len(),
            children: HashMap::new(),
            content,
        }
    }
}

pub struct FileSystem {
    pub root: Node,
    pub current_path: Vec<String>,
    pub total_space: usize,
    pub used_space: usize,
}

impl FileSystem {
    pub fn new(size_mb: usize) -> Self {
        let mut fs = Self {
            root: Node::new_dir("/"),
            current_path: Vec::new(),
            total_space: size_mb * 1024 * 1024,
            used_space: 0,
        };
        // Initialize standard directories
        fs.mkdir_internal("local").unwrap();
        fs.mkdir_internal("tmp").unwrap();
        fs.mkdir_internal("bin").unwrap();

        // Preload hello.wasm
        let hello_wasm = include_bytes!(r"../../apps/hello/target/wasm32-unknown-unknown/release/hello.wasm");
        if let Some(bin) = fs.root.children.get_mut("bin") {
             bin.children.insert("hello.wasm".to_string(), Node::new_file("hello.wasm", hello_wasm.to_vec()));
        }

        // Preload math.wasm
        let math_wasm = include_bytes!(r"../../apps/math/target/wasm32-unknown-unknown/release/math.wasm");
         if let Some(bin) = fs.root.children.get_mut("bin") {
             bin.children.insert("math.wasm".to_string(), Node::new_file("math.wasm", math_wasm.to_vec()));
        }

        // Preload desktop.wasm
        let desktop_wasm = include_bytes!(r"../../apps/desktop/target/wasm32-unknown-unknown/release/desktop.wasm");
         if let Some(bin) = fs.root.children.get_mut("bin") {
             bin.children.insert("desktop.wasm".to_string(), Node::new_file("desktop.wasm", desktop_wasm.to_vec()));
        }

        // Preload terminal.wasm
        let terminal_wasm = include_bytes!(r"../../apps/terminal/target/wasm32-unknown-unknown/release/terminal.wasm");
         if let Some(bin) = fs.root.children.get_mut("bin") {
             bin.children.insert("terminal.wasm".to_string(), Node::new_file("terminal.wasm", terminal_wasm.to_vec()));
        }
        
        // Load persistent storage for "local"
        fs.load_local_disk();
        
        fs
    }

    // Helper to get window.localStorage
    fn get_storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;
        window.local_storage().ok().flatten()
    }
    
    fn save_local_disk(&self) {
        if let Some(storage) = Self::get_storage() {
            if let Some(local_node) = self.root.children.get("local") {
                if let Ok(json) = serde_json::to_string(local_node) {
                    let _ = storage.set_item("wasmix_fs_local", &json);
                }
            }
        }
    }

    fn load_local_disk(&mut self) {
        if let Some(storage) = Self::get_storage() {
             if let Ok(Some(json)) = storage.get_item("wasmix_fs_local") {
                 if let Ok(node) = serde_json::from_str::<Node>(&json) {
                     self.root.children.insert("local".to_string(), node);
                 }
             }
        }
    }

    // Internal mkdir without saving (used during init)
    fn mkdir_internal(&mut self, path: &str) -> Result<(), String> {
        if self.used_space + 4096 > self.total_space { 
            return Err("Disk full".to_string());
        }

        let path_clone = self.current_path.clone();
        let target_dir = self.resolve_mut_dir(&path_clone)?;
        if target_dir.children.contains_key(path) {
            return Err("Directory exists".to_string());
        }

        target_dir.children.insert(path.to_string(), Node::new_dir(path));
        self.used_space += 4096;
        Ok(())
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), String> {
        let res = self.mkdir_internal(path);
        if res.is_ok() {
            // If we are in /local or root creating local (handled by internal but general check)
            // Simplest check: check if modified path affects local
            // Ideally we check if absolute path starts with /local
            // But strict path checking is complex here.
            // Let's just save all the time for MVP safety or check if current path is empty (root) and target is local? No user won't create local.
            // Check if we are inside local
            if self.is_inside_local() {
                self.save_local_disk();
            }
        }
        res
    }

    pub fn create_file(&mut self, path: &str) -> Result<(), String> {
        if self.used_space + 10 > self.total_space { 
            return Err("Disk full".to_string());
        }

        let path_clone = self.current_path.clone();
        let target_dir = self.resolve_mut_dir(&path_clone)?;
        if target_dir.children.contains_key(path) {
             return Err("File or Directory exists".to_string());
        }

        target_dir.children.insert(path.to_string(), Node::new_file(path, Vec::new()));
        self.used_space += 10; 
        
        if self.is_inside_local() {
            self.save_local_disk();
        }
        
        Ok(())
    }

    pub fn remove_entry(&mut self, path: &str) -> Result<(), String> {
        let path_clone = self.current_path.clone();
        let target_dir = self.resolve_mut_dir(&path_clone)?;
        
        if let Some(node) = target_dir.children.remove(path) {
             // Reclaim space (Simplified)
             self.used_space = self.used_space.saturating_sub(node.size);
             if let NodeType::Directory = node.node_type {
                 self.used_space = self.used_space.saturating_sub(4096);
             }
             
             if self.is_inside_local() {
                 self.save_local_disk();
             }
             Ok(())
        } else {
             Err("File or directory not found".to_string())
        }
    }
    
    fn is_inside_local(&self) -> bool {
        // If current path starts with "local"
        if !self.current_path.is_empty() && self.current_path[0] == "local" {
            return true;
        }
        false
    }

    pub fn cd(&mut self, path: &str) -> Result<(), String> {
        if path == "/" {
            self.current_path.clear();
            return Ok(());
        }
        if path == ".." {
            self.current_path.pop();
            return Ok(());
        }

        // Check if directory exists in current directory
        // Validation logic
        // We need to resolve the current directory first to check children
        // We can't borrow self mutable and immutable at same time easily if we strictly use helpers
        // But resolve_dir is using &self, correct.
        
        let path_clone = self.current_path.clone(); // Clone to avoid borrow conflict if we need mut self later, though here strictly we need to check existence first.
        
        let dir_node = self.resolve_dir(&path_clone).ok_or("Current path invalid")?;
        
        if let Some(child) = dir_node.children.get(path) {
            if let NodeType::Directory = child.node_type {
                 self.current_path.push(path.to_string());
                 Ok(())
            } else {
                Err("Not a directory".to_string())
            }
        } else {
             Err("Directory not found".to_string())
        }
    }

    pub fn list_dir(&self) -> Vec<String> {
        let dir = self.resolve_dir(&self.current_path).unwrap_or(&self.root);
        let mut names: Vec<String> = dir.children.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn match_entry(&self, pattern: &str) -> Option<String> {
        let dir = self.resolve_dir(&self.current_path).unwrap_or(&self.root);
        
        // Direct match first
        if dir.children.contains_key(pattern) {
            return Some(pattern.to_string());
        }

        // Wildcard match
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            for name in dir.children.keys() {
                if name.starts_with(prefix) {
                    return Some(name.clone());
                }
            }
        }
        
        None
    }
    
    // Helpers
    pub fn resolve_dir(&self, path: &[String]) -> Option<&Node> {
        let mut current_node = &self.root;
        for part in path {
            if let Some(node) = current_node.children.get(part) {
                if let NodeType::Directory = node.node_type {
                    current_node = node;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(current_node)
    }

    fn resolve_mut_dir(&mut self, path: &[String]) -> Result<&mut Node, String> {
        let mut current_node = &mut self.root;
        for part in path {
            // This requires re-borrowing which is tricky in a loop.
            // A common pattern for simple trees is to keep path or use indexes/Arena.
            // For MVP with deep recursion, we can just "drill down" if the borrow checker allows, 
            // but loop borrow is hard.
            // REWRITE: Simple "flat" map of paths? Or just support 1 level deep for MVP?
            // To properly do tree traversal mutably: define a recursive helper or use Unsafe / Raw pointers.
            // Or... since this is single threaded WASM, we can be a bit sloppy, but let's be safe.
            // Alternative: `get_mut` on HashMap returns mutable ref, we can chain them?
            // `current_node = current_node.children.get_mut(part)` works if we don't hold the previous ref.
            
            if let Some(node) = current_node.children.get_mut(part) {
                if let NodeType::Directory = node.node_type {
                    current_node = node;
                } else {
                    return Err(format!("{} is not a directory", part));
                }
            } else {
                return Err(format!("Directory {} not found", part));
            }
        }
        Ok(current_node)
    }
}
