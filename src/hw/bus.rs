use super::ram::Ram;
use super::gpu::Gpu;

#[allow(dead_code)]
pub struct Bus {
    pub ram: Ram,
    pub gpu: Gpu,
}

impl Bus {
    pub fn new(ram: Ram, gpu: Gpu) -> Self {
        Self {
            ram,
            gpu,
        }
    }
    
    // Future: read/write mapping
    // 0x00000000 - 0x0FFFFFFF -> RAM (256MB window commonly, we use what we have)
    // 0xA0000000 - ... -> VRAM
}
