use rusttype::{point, Font, PositionedGlyph, Scale};
use geom::Size;

fn calc_text_width(glyphs: &[PositionedGlyph]) -> f32 {
    glyphs.last().unwrap().pixel_bounding_box().unwrap().max.x as f32
}

pub fn text_to_texture(font: &Font, height: f32, text: &str) -> (Size<i32>, Vec<u8>) {
    let scale = Scale {
        x: height,
        y: height,
    };
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);
    let glyphs: Vec<_> = font.layout(text, scale, offset).collect();
    let pixel_height = height.ceil() as usize;
    let width = calc_text_width(&glyphs) as usize;
    let mut pixel_data = vec![0_u8; 4 * width * pixel_height];
    let mapping_scale = 255.0;
    for g in glyphs {
        let bb = match g.pixel_bounding_box() {
            Some(bb) => bb,
            None => continue,
        };
        g.draw(|x, y, v| {
            let v = (v * mapping_scale + 0.5) as u8;
            let x = x as i32 + bb.min.x;
            let y = y as i32 + bb.min.y;
            // There's still a possibility that the glyph clips the boundaries of the bitmap
            if v > 0 && x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                let i = (x as usize + y as usize * width) * 4;
                pixel_data[i] = 255;
                pixel_data[i + 1] = 255;
                pixel_data[i + 2] = 255;
                pixel_data[i + 3] = v;
            }
        });
    }
    let size = Size {
        w: width as i32,
        h: pixel_height as i32,
    };
    (size, pixel_data)
}
