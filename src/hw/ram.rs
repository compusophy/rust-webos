#![allow(dead_code)]
pub struct Ram {
    pub mem: Vec<u8>,
    pub size: usize,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            mem: vec![0; size],
            size,
        }
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        if (addr as usize) < self.size {
            self.mem[addr as usize]
        } else {
            0
        }
    }

    pub fn write_u8(&mut self, addr: u32, val: u8) {
        if (addr as usize) < self.size {
            self.mem[addr as usize] = val;
        }
    }
    
    // Helper helpers
    pub fn read_u32(&self, addr: u32) -> u32 {
        let addr = addr as usize;
        if addr + 4 <= self.size {
            let _b0 = self.mem[addr] as u32;
            let _b1 = self.mem[addr+1] as u32;
            let _b2 = self.mem[addr+2] as u32;
            let _b3 = self.mem[addr+3] as u32;
            u32::from_le_bytes([self.mem[addr], self.mem[addr+1], self.mem[addr+2], self.mem[addr+3]])
        } else {
            0
        }
    }

    pub fn write_u32(&mut self, addr: u32, val: u32) {
         let addr = addr as usize;
         if addr + 4 <= self.size {
             let bytes = val.to_le_bytes();
             self.mem[addr] = bytes[0];
             self.mem[addr+1] = bytes[1];
             self.mem[addr+2] = bytes[2];
             self.mem[addr+3] = bytes[3];
         }
    }
}
