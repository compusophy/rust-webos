pub struct Gpu {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>, // VRAM: Simple linear framebuffer for now
}

impl Gpu {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let buffer = vec![255; size]; 
        Self { width, height, buffer }
    }

    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        for chunk in self.buffer.chunks_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 255; 
        }
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        
        // Color format: 0xAABBGGRR
        let r = ((color >> 24) & 0xFF) as u8;
        let g = ((color >> 16) & 0xFF) as u8;
        let b = ((color >> 8) & 0xFF) as u8;
        let a = (color & 0xFF) as u8;

        self.buffer[idx] = r;
        self.buffer[idx + 1] = g;
        self.buffer[idx + 2] = b;
        self.buffer[idx + 3] = a;
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        for iy in y..(y + h) {
            for ix in x..(x + w) {
                self.put_pixel(ix, iy, color);
            }
        }
    }
}
