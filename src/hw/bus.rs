use std::rc::Rc;
use std::cell::RefCell;
use super::ram::Ram;
use super::gpu::Gpu;

#[allow(dead_code)]
pub struct Bus {
    pub ram: Rc<RefCell<Ram>>,
    pub gpu: Rc<RefCell<Gpu>>,
}

impl Bus {
    pub fn new(ram: Rc<RefCell<Ram>>, gpu: Rc<RefCell<Gpu>>) -> Self {
        Self {
            ram,
            gpu,
        }
    }
    
    // Future: read/write mapping
    // 0x00000000 - 0x0FFFFFFF -> RAM (256MB window commonly, we use what we have)
    // 0xA0000000 - ... -> VRAM
}
