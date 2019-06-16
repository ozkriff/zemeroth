use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteInfo {
    pub paths: HashMap<String, String>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub shadow_size_coefficient: f32,
}
