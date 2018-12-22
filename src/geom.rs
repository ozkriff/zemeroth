use ggez::graphics::{Point2, Vector2};
use rand::{thread_rng, Rng};

use crate::core::map::{hex_round, PosHex};

const SQRT_OF_3: f32 = 1.732_05;

pub const FLATNESS_COEFFICIENT: f32 = 0.8;

/// <http://www.redblobgames.com/grids/hexagons/#hex-to-pixel>
pub fn hex_to_point(size: f32, hex: PosHex) -> Point2 {
    let x = size * SQRT_OF_3 * (hex.q as f32 + hex.r as f32 / 2.0);
    let y = size * 3.0 / 2.0 * hex.r as f32;
    Point2::new(x, y * FLATNESS_COEFFICIENT)
}

/// <http://www.redblobgames.com/grids/hexagons/#pixel-to-hex>
pub fn point_to_hex(size: f32, mut point: Point2) -> PosHex {
    point.y /= FLATNESS_COEFFICIENT;
    let q = (point.x * SQRT_OF_3 / 3.0 - point.y / 3.0) / size;
    let r = point.y * 2.0 / 3.0 / size;
    hex_round(PosHex { q, r })
}

pub fn rand_tile_offset(size: f32, radius: f32) -> Vector2 {
    assert!(radius >= 0.0);
    let r = size * radius;
    Vector2::new(
        thread_rng().gen_range(-r, r),
        thread_rng().gen_range(-r, r) * FLATNESS_COEFFICIENT,
    )
}
