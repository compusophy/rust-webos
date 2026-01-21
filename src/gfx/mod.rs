pub mod font;

pub struct Context {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
}

impl Context {
    pub fn new(width: u32, height: u32) -> Self {
        // RGBA buffer
        let size = (width * height * 4) as usize;
        let buffer = vec![255; size]; // Start white or black
        Self { width, height, buffer }
    }

    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        for chunk in self.buffer.chunks_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 255; // Alpha
        }
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        // Color is 0xAABBGGRR (little endian u32) or we can decide.
        // Let's assume input is 0xAABBGGRR for easy hex usage 0xFF_00_00_FF (Red full alpha)
        // But u32 in rust is big/little endian dependent.
        // Let's decompose manually to be safe.
        // Format: R G B A
        
        // Let's interpret 'color' as 0xRRGGBBAA
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
