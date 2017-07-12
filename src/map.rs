use cgmath::vec2;
use hate::geom::Point;
use core::map::{PosHex, hex_round};

const SQRT_OF_3: f32 = 1.732050;

/// http://www.redblobgames.com/grids/hexagons/#hex-to-pixel
pub fn hex_to_point(size: f32, hex: PosHex) -> Point {
    let x = size * SQRT_OF_3 * (hex.q as f32 + hex.r as f32 / 2.0);
    let y = size * 3.0 / 2.0 * hex.r as f32;
    Point(vec2(x, y))
}

/// http://www.redblobgames.com/grids/hexagons/#pixel-to-hex
pub fn point_to_hex(size: f32, point: Point) -> PosHex {
    let q = (point.0.x * SQRT_OF_3 / 3.0 - point.0.y / 3.0) / size;
    let r = point.0.y * 2.0 / 3.0 / size;
    hex_round(PosHex { q, r })
}
