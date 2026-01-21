use crate::gfx::Context;
use font8x8::{BASIC_FONTS, UnicodeFonts};

pub fn draw_char(ctx: &mut Context, x: u32, y: u32, c: char, color: u32) {
    // We use BASIC_FONTS which covers ASCII
    if let Some(glyph) = BASIC_FONTS.get(c) {
         for (row_i, byte) in glyph.iter().enumerate() {
            // font8x8: byte is a row
            // "The least significant bit corresponds to the column with the lowest index" 
            // from some docs: "Bit 0 is the first column".
            for col_i in 0..8 {
                if (byte & (1 << col_i)) != 0 {
                    ctx.put_pixel(x + col_i as u32, y + row_i as u32, color);
                }
            }
        }
    } else {
        // Draw a box for missing char
        for i in 0..8 {
            ctx.put_pixel(x + i, y, color);
            ctx.put_pixel(x + i, y + 7, color);
            ctx.put_pixel(x, y + i, color);
            ctx.put_pixel(x + 7, y + i, color);
        }
    }
}
