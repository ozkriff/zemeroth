use macroquad::prelude::{vec2, Vec2};

use crate::core::{
    map::{hex_round, PosHex},
    utils::roll_dice,
};

const SQRT_OF_3: f32 = 1.732_05;

pub const FLATNESS_COEFFICIENT: f32 = 0.8125; // should fit the tile sprite's geometry

/// <http://www.redblobgames.com/grids/hexagons/#hex-to-pixel>
pub fn hex_to_point(size: f32, hex: PosHex) -> Vec2 {
    let x = size * SQRT_OF_3 * (hex.q as f32 + hex.r as f32 / 2.0);
    let y = size * 3.0 / 2.0 * hex.r as f32;
    Vec2::new(x, y * FLATNESS_COEFFICIENT)
}

/// <http://www.redblobgames.com/grids/hexagons/#pixel-to-hex>
pub fn point_to_hex(size: f32, mut point: Vec2) -> PosHex {
    *point.y_mut() /= FLATNESS_COEFFICIENT;
    let q = (point.x() * SQRT_OF_3 / 3.0 - point.y() / 3.0) / size;
    let r = point.y() * 2.0 / 3.0 / size;
    hex_round(PosHex { q, r })
}

pub fn rand_tile_offset(size: f32, radius: f32) -> Vec2 {
    assert!(radius >= 0.0);
    let r = size * radius;
    Vec2::new(roll_dice(-r, r), roll_dice(-r, r) * FLATNESS_COEFFICIENT)
}

#[derive(Clone, Copy, Debug)]
pub enum Facing {
    Left,
    Right,
}

impl Facing {
    pub fn from_positions(tile_size: f32, from: PosHex, to: PosHex) -> Option<Self> {
        if from == to {
            return None;
        }
        let from = hex_to_point(tile_size, from);
        let to = hex_to_point(tile_size, to);
        Some(if to.x() > from.x() {
            Facing::Right
        } else {
            Facing::Left
        })
    }

    pub fn to_scene_facing(self) -> zscene::Facing {
        match self {
            Facing::Left => zscene::Facing::Left,
            Facing::Right => zscene::Facing::Right,
        }
    }
}
